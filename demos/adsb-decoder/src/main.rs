use anyhow::Result;
use clap::Parser;
use num_complex::Complex;
use sdr::source::parse_source_arg;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod decode;
mod demod;

use decode::{AdsbMessage, Payload, decode_cpr_global, decode_message};
use demod::AdsbDemodulator;

#[derive(Parser)]
#[command(name = "adsb-decoder", about = "Decode ADS-B aircraft messages from 1090 MHz")]
struct Args {
    /// SDR source: file:PATH, tcp:HOST:PORT, or usb[:INDEX]
    #[arg(short, long, default_value = "tcp:127.0.0.1:1234")]
    source: String,

    /// Sample rate in Hz (must be >= 2 MS/s)
    #[arg(long, default_value_t = 2_400_000)]
    sample_rate: u32,

    /// Tuner gain in dB. Omit for auto gain.
    #[arg(short, long)]
    gain: Option<f64>,
}

/// Tracked state for a single aircraft.
struct Aircraft {
    icao: u32,
    callsign: Option<String>,
    altitude_ft: Option<i32>,
    lat: Option<f64>,
    lon: Option<f64>,
    ground_speed_kt: Option<u16>,
    vertical_rate_fpm: Option<i32>,
    heading: Option<f32>,
    messages: u32,
    // CPR state for position decoding
    cpr_even: Option<(u32, u32)>,
    cpr_odd: Option<(u32, u32)>,
    last_cpr_odd: bool,
}

impl Aircraft {
    fn new(icao: u32) -> Self {
        Self {
            icao,
            callsign: None,
            altitude_ft: None,
            lat: None,
            lon: None,
            ground_speed_kt: None,
            vertical_rate_fpm: None,
            heading: None,
            messages: 0,
            cpr_even: None,
            cpr_odd: None,
            last_cpr_odd: false,
        }
    }

    fn update(&mut self, msg: &AdsbMessage) {
        self.messages += 1;
        match &msg.payload {
            Payload::Identification { callsign } => {
                self.callsign = Some(callsign.clone());
            }
            Payload::Position {
                altitude_ft,
                cpr_lat,
                cpr_lon,
                cpr_odd,
            } => {
                if let Some(alt) = altitude_ft {
                    self.altitude_ft = Some(*alt);
                }
                if *cpr_odd {
                    self.cpr_odd = Some((*cpr_lat, *cpr_lon));
                    self.last_cpr_odd = true;
                } else {
                    self.cpr_even = Some((*cpr_lat, *cpr_lon));
                    self.last_cpr_odd = false;
                }
                // Try to resolve position if we have both even and odd
                if let (Some((lat_e, lon_e)), Some((lat_o, lon_o))) =
                    (self.cpr_even, self.cpr_odd)
                {
                    if let Some((lat, lon)) =
                        decode_cpr_global(lat_e, lon_e, lat_o, lon_o, self.last_cpr_odd)
                    {
                        self.lat = Some(lat);
                        self.lon = Some(lon);
                    }
                }
            }
            Payload::Velocity {
                ground_speed_kt,
                vertical_rate_fpm,
                heading,
            } => {
                if ground_speed_kt.is_some() {
                    self.ground_speed_kt = *ground_speed_kt;
                }
                if vertical_rate_fpm.is_some() {
                    self.vertical_rate_fpm = *vertical_rate_fpm;
                }
                if heading.is_some() {
                    self.heading = *heading;
                }
            }
            Payload::Other { .. } => {}
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop))
        .expect("registering SIGINT handler");

    let freq_hz: u32 = 1_090_000_000;

    println!("ADS-B Decoder — 1090 MHz");
    println!("  Sample rate: {} MS/s", args.sample_rate as f64 / 1e6);
    println!("  Source: {}", args.source);
    println!();

    let config = parse_source_arg(&args.source, args.sample_rate)?;
    let mut source = sdr::open_source(config)?;
    source.set_frequency(freq_hz)?;
    source.set_sample_rate(args.sample_rate)?;

    if let Some(gain) = args.gain {
        source.set_gain(Some((gain * 10.0) as i32))?;
        println!("  Gain: {gain:.1} dB");
    }

    let demod = AdsbDemodulator::new(args.sample_rate);
    let mut aircraft: HashMap<u32, Aircraft> = HashMap::new();
    let mut total_messages: u64 = 0;
    let mut total_preambles: u64 = 0;
    let mut total_crc_fail: u64 = 0;
    let mut read_count: u64 = 0;

    println!("Listening for aircraft... Press Ctrl-C to stop.");
    println!();
    println!(
        "{:>6}  {:>8}  {:>8}  {:>8}  {:>8}  {:>5}  {:>6}  {:>5}",
        "ICAO", "Callsign", "Alt (ft)", "Lat", "Lon", "Spd", "V/S", "Msgs"
    );
    println!("{}", "─".repeat(72));

    let buf_size = 256 * 1024;
    let mut iq_buf = vec![Complex::new(0.0f32, 0.0); buf_size];

    while !stop.load(Ordering::Relaxed) {
        let n = source.read(&mut iq_buf)?;
        if n == 0 {
            println!("End of input.");
            break;
        }
        read_count += 1;

        // Compute magnitude (AM envelope)
        let mag: Vec<f32> = iq_buf[..n].iter().map(|s| s.norm()).collect();

        // Detect and extract messages
        let raw_messages = demod.process(&mag);
        total_preambles += raw_messages.len() as u64;

        for bits in &raw_messages {
            if let Some(msg) = decode_message(bits) {
                total_messages += 1;

                let ac = aircraft
                    .entry(msg.icao)
                    .or_insert_with(|| Aircraft::new(msg.icao));
                ac.update(&msg);

                // Print updated aircraft info
                print_aircraft(ac);
            } else {
                total_crc_fail += 1;
            }
        }

        // Periodic status every 500 reads
        if read_count % 500 == 0 {
            eprintln!(
                "[status] {total_messages} decoded, {} aircraft tracked",
                aircraft.len()
            );
        }
    }

    println!();
    println!(
        "Done. {} messages decoded, {} aircraft seen.",
        total_messages,
        aircraft.len()
    );

    Ok(())
}

fn print_aircraft(ac: &Aircraft) {
    let callsign = ac
        .callsign
        .as_deref()
        .unwrap_or("--------");
    let alt = ac
        .altitude_ft
        .map(|a| format!("{:>7}", a))
        .unwrap_or_else(|| "      -".to_string());
    let lat = ac
        .lat
        .map(|l| format!("{:>8.3}", l))
        .unwrap_or_else(|| "       -".to_string());
    let lon = ac
        .lon
        .map(|l| format!("{:>8.3}", l))
        .unwrap_or_else(|| "       -".to_string());
    let spd = ac
        .ground_speed_kt
        .map(|s| format!("{:>5}", s))
        .unwrap_or_else(|| "    -".to_string());
    let vs = ac
        .vertical_rate_fpm
        .map(|v| format!("{:>+6}", v))
        .unwrap_or_else(|| "     -".to_string());

    println!(
        "{:06X}  {:>8}  {}  {}  {}  {}  {}  {:>5}",
        ac.icao, callsign, alt, lat, lon, spd, vs, ac.messages
    );
}
