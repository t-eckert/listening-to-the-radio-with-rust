use anyhow::Result;
use clap::Parser;
use num_complex::Complex;
use sdr::source::parse_source_arg;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Parser)]
#[command(name = "iq-print", about = "Print raw IQ samples from an SDR source")]
struct Args {
    /// SDR source: file:PATH, tcp:HOST:PORT, or usb[:INDEX]
    #[arg(short, long, default_value = "tcp:127.0.0.1:1234")]
    source: String,

    /// Center frequency in MHz
    #[arg(short, long, default_value_t = 100.3)]
    frequency: f64,

    /// Sample rate in Hz
    #[arg(long, default_value_t = 2_048_000)]
    sample_rate: u32,

    /// Number of samples to print per batch
    #[arg(short, long, default_value_t = 10)]
    count: usize,

    /// Delay between batches in milliseconds
    #[arg(short, long, default_value_t = 500)]
    delay: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop))
        .expect("registering SIGINT handler");

    let freq_hz = (args.frequency * 1e6) as u32;

    println!("IQ Sample Viewer");
    println!("  Frequency: {:.1} MHz", args.frequency);
    println!("  Sample rate: {} Hz", args.sample_rate);
    println!("  Source: {}", args.source);
    println!();

    let config = parse_source_arg(&args.source, args.sample_rate)?;
    let mut source = sdr::open_source(config)?;
    source.set_frequency(freq_hz)?;
    source.set_sample_rate(args.sample_rate)?;

    let mut buf = vec![Complex::new(0.0f32, 0.0); 4096];

    println!("{:>6}  {:>10}  {:>10}  {:>10}  {:>8}", "n", "I", "Q", "|mag|", "phase");
    println!("{}", "─".repeat(54));

    let mut sample_num: u64 = 0;

    while !stop.load(Ordering::Relaxed) {
        let n = source.read(&mut buf)?;
        if n == 0 {
            println!("End of input.");
            break;
        }

        for sample in &buf[..n.min(args.count)] {
            let mag = sample.norm();
            let phase = sample.im.atan2(sample.re);
            println!(
                "{:>6}  {:>10.6}  {:>10.6}  {:>10.6}  {:>8.4}",
                sample_num, sample.re, sample.im, mag, phase
            );
            sample_num += 1;
        }

        sample_num += (n - n.min(args.count)) as u64;
        println!();

        std::thread::sleep(std::time::Duration::from_millis(args.delay));
    }

    Ok(())
}
