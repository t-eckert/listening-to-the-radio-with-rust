use anyhow::Result;
use clap::Parser;
use num_complex::Complex;
use sdr::dsp::{am::AmDemodulator, filter::LowPassFilter, tone::GoertzelDetector};
use sdr::source::parse_source_arg;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod decoder;
mod display;

use decoder::ChuDecoder;

#[derive(Parser)]
#[command(name = "chu-decoder", about = "Decode CHU Ottawa time signal")]
struct Args {
    /// SDR source: file:PATH, tcp:HOST:PORT, or usb[:INDEX]
    #[arg(short, long, default_value = "tcp:127.0.0.1:1234")]
    source: String,

    /// CHU frequency in Hz (default: 7850000 for best Ottawa reception)
    #[arg(short, long, default_value_t = 7_850_000)]
    frequency: u32,

    /// Sample rate in Hz
    #[arg(long, default_value_t = 1_024_000)]
    sample_rate: u32,

    /// Tuner gain in dB (e.g., 49.0). Omit for auto gain.
    #[arg(short, long)]
    gain: Option<f64>,

    /// Enable direct sampling for shortwave reception below ~24 MHz.
    /// Use Q-branch (2) for the RTL-SDR Blog V4.
    #[arg(long, default_value_t = 2)]
    direct_sampling: u32,

    /// Skip sending setup commands to the SDR source.
    /// Use when rtl_tcp is already configured with the correct settings.
    #[arg(long)]
    no_setup: bool,

    /// Show diagnostic output (tone power levels)
    #[arg(long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop))
        .expect("registering SIGINT handler");

    let freq_mhz = args.frequency as f64 / 1e6;
    println!("CHU Time Signal Decoder");
    println!("  Frequency: {freq_mhz:.3} MHz");
    println!("  Sample rate: {} Hz", args.sample_rate);
    println!("  Source: {}", args.source);

    let config = parse_source_arg(&args.source, args.sample_rate)?;
    let mut source = sdr::open_source(config)?;

    if args.no_setup {
        println!("  Skipping SDR setup (--no-setup)");
    } else if args.direct_sampling > 0 {
        // For direct sampling via rtl_tcp: set freq first, then enable
        // direct sampling LAST. Do NOT send sample_rate — it resets the
        // device and clears direct sampling mode.
        source.set_frequency(args.frequency)?;
        source.set_direct_sampling(args.direct_sampling)?;
        println!("  Direct sampling: mode {}", args.direct_sampling);

        // Wait for direct sampling to take effect, then flush stale data
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut flush_buf = vec![Complex::new(0.0f32, 0.0); 16384];
        for _ in 0..20 {
            let _ = source.read(&mut flush_buf);
        }
    } else {
        source.set_frequency(args.frequency)?;
        source.set_sample_rate(args.sample_rate)?;
        let gain = args.gain.unwrap_or(49.6);
        source.set_gain(Some((gain * 10.0) as i32))?;
        println!("  Gain: {gain:.1} dB");
    }
    println!();

    let sdr_rate = args.sample_rate;

    // DSP chain for CHU:
    // CHU transmits FSK at 2025/2225 Hz (Bell 103 style, 300 baud)
    // 2225 Hz = mark (1), 2025 Hz = space (0)
    //
    // 1. AM demodulate to get the audio envelope
    let am_demod = AmDemodulator::new();

    // 2. Low-pass filter + decimate to a manageable audio rate
    let audio_rate = 8000u32;
    let audio_cutoff = 3500.0 / sdr_rate as f32;
    let decimation = (sdr_rate / audio_rate) as usize;
    let mut audio_filter = LowPassFilter::new(audio_cutoff, 101, decimation);

    // 3. Goertzel detectors for the FSK tones and the 1000 Hz tick
    let mark_detector = GoertzelDetector::new(2225.0, audio_rate as f32);
    let space_detector = GoertzelDetector::new(2025.0, audio_rate as f32);
    let tick_detector = GoertzelDetector::new(1000.0, audio_rate as f32);

    // 4. CHU protocol decoder
    let mut chu_decoder = ChuDecoder::new();

    println!("Listening for CHU time signals... Press Ctrl-C to stop.");
    println!();

    let buf_size = 16384;
    let mut iq_buf = vec![Complex::new(0.0f32, 0.0); buf_size];

    // Process in blocks matching the baud rate
    // CHU sends at 300 baud → ~26.67 samples per bit at 8000 Hz
    let samples_per_bit = (audio_rate as f32 / 300.0) as usize;
    let mut audio_accumulator: Vec<f32> = Vec::new();
    let mut tick_count = 0u64;
    let mut bit_count = 0u64;
    let mut block_count = 0u64;

    while !stop.load(Ordering::Relaxed) {
        let n = source.read(&mut iq_buf)?;
        if n == 0 {
            println!("End of input.");
            break;
        }
        let iq = &iq_buf[..n];

        // Debug: print raw IQ signal level every ~1 second
        block_count += 1;
        if args.debug && block_count % 60 == 1 {
            let max_mag = iq.iter().map(|s| s.norm()).fold(0.0f32, f32::max);
            let avg_mag: f32 = iq.iter().map(|s| s.norm()).sum::<f32>() / n as f32;
            println!(
                "  [raw IQ] block={block_count} samples={n} avg_mag={avg_mag:.6} max_mag={max_mag:.6}"
            );
        }

        // AM demodulate
        let envelope = am_demod.process_ac_coupled(iq);

        // Filter and decimate to audio rate
        let audio = audio_filter.process_real(&envelope);

        // Accumulate audio and process bit-by-bit
        audio_accumulator.extend_from_slice(&audio);

        while audio_accumulator.len() >= samples_per_bit {
            let bit_samples = &audio_accumulator[..samples_per_bit];

            // Detect tones
            let mark_power = mark_detector.power(bit_samples);
            let space_power = space_detector.power(bit_samples);
            let tick_power = tick_detector.power(bit_samples);

            // Check for 1000 Hz tick (much stronger than FSK tones when present)
            let fsk_power = mark_power + space_power;
            let is_tick = tick_power > fsk_power * 2.0 && tick_power > 1.0;

            if is_tick {
                tick_count += 1;
                if args.debug && tick_count % 10 == 1 {
                    println!(
                        "  tick #{tick_count}  (1000Hz: {tick_power:.0}, FSK: {fsk_power:.0})"
                    );
                }
            }

            // FSK bit decision
            let bit = if mark_power > space_power { 1u8 } else { 0u8 };
            bit_count += 1;

            if args.debug && bit_count % 200 == 0 {
                println!(
                    "  bits: {bit_count} | mark: {mark_power:.1} space: {space_power:.1} tick: {tick_power:.1} | bit={bit}"
                );
            }

            if let Some(time_info) = chu_decoder.feed_bit(bit) {
                display::print_time(&time_info);
            }

            audio_accumulator.drain(..samples_per_bit);
        }
    }

    println!("\nShutting down.");
    println!(
        "Stats: {bit_count} bits processed, {tick_count} ticks detected"
    );
    Ok(())
}
