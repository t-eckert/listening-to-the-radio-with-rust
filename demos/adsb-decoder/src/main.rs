use anyhow::Result;
use clap::Parser;
use num_complex::Complex;
use sdr::source::parse_source_arg;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

mod db;
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

    /// SQLite database path for storing aircraft positions
    #[arg(long, default_value = "adsb.db")]
    db: String,
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
    cpr_even: Option<(u32, u32, Instant)>,
    cpr_odd: Option<(u32, u32, Instant)>,
    last_cpr_odd: bool,
    /// Set to true when a new position fix is resolved, cleared after DB write
    new_position: bool,
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
            new_position: false,
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
                let now = Instant::now();
                if *cpr_odd {
                    self.cpr_odd = Some((*cpr_lat, *cpr_lon, now));
                    self.last_cpr_odd = true;
                } else {
                    self.cpr_even = Some((*cpr_lat, *cpr_lon, now));
                    self.last_cpr_odd = false;
                }
                // Try to resolve position if we have both even and odd
                // Only use the pair if both frames are less than 10 seconds old
                if let (Some((lat_e, lon_e, t_e)), Some((lat_o, lon_o, t_o))) =
                    (self.cpr_even, self.cpr_odd)
                {
                    let age = if self.last_cpr_odd {
                        now.duration_since(t_e)
                    } else {
                        now.duration_since(t_o)
                    };
                    if age.as_secs() < 10 {
                        if let Some((lat, lon)) =
                            decode_cpr_global(lat_e, lon_e, lat_o, lon_o, self.last_cpr_odd)
                        {
                            self.lat = Some(lat);
                            self.lon = Some(lon);
                            self.new_position = true;
                        }
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

    match args.gain {
        Some(gain) => {
            source.set_gain(Some((gain * 10.0) as i32))?;
            println!("  Gain: {gain:.1} dB");
        }
        None => {
            // ADS-B is a handful of 120 μs bursts per second on an otherwise
            // empty channel. The tuner's AGC spends nearly all its time looking
            // at noise, settles low, and never opens up in time for a burst, so
            // auto gain misses aircraft that max gain hears easily. Default to
            // the highest gain the tuner actually reports.
            match source.max_gain()? {
                Some(g) => {
                    source.set_gain(Some(g))?;
                    println!("  Gain: {:.1} dB (tuner maximum)", g as f64 / 10.0);
                }
                None => println!("  Gain: auto"),
            }
        }
    }

    let db = db::AdsbDb::open(&args.db)?;
    println!("  Database: {}", args.db);

    let demod = AdsbDemodulator::new(args.sample_rate);
    let mut aircraft: HashMap<u32, Aircraft> = HashMap::new();
    let mut verified_icaos: HashSet<u32> = HashSet::new(); // ICAOs confirmed by DF17/18 CRC
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
                // DF17/18 are CRC-validated — always trust them
                if msg.df == 17 || msg.df == 18 {
                    verified_icaos.insert(msg.icao);
                } else if !verified_icaos.contains(&msg.icao) {
                    // Short messages: only accept if ICAO was seen in a DF17/18
                    total_crc_fail += 1;
                    continue;
                }

                total_messages += 1;

                let ac = aircraft
                    .entry(msg.icao)
                    .or_insert_with(|| Aircraft::new(msg.icao));
                ac.update(&msg);

                // Write to database
                let _ = db.update_aircraft(
                    ac.icao,
                    ac.callsign.as_deref(),
                    ac.messages,
                );
                if ac.new_position {
                    if let (Some(lat), Some(lon)) = (ac.lat, ac.lon) {
                        let _ = db.insert_position(
                            ac.icao,
                            lat,
                            lon,
                            ac.altitude_ft,
                            ac.ground_speed_kt,
                            ac.vertical_rate_fpm,
                            ac.heading,
                        );
                    }
                    ac.new_position = false;
                }

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
