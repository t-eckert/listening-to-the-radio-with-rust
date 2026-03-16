use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use num_complex::Complex;
use ratatui::prelude::*;
use sdr::source::parse_source_arg;
use std::io::stdout;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

mod scanner;
mod ui;

use scanner::FrequencyScanner;

#[derive(Parser)]
#[command(name = "freq-scanner", about = "Frequency scanner with spectrum display")]
struct Args {
    /// SDR source: file:PATH, tcp:HOST:PORT, or usb[:INDEX]
    #[arg(short, long, default_value = "tcp:127.0.0.1:1234")]
    source: String,

    /// Start frequency in MHz
    #[arg(long, default_value_t = 88.0)]
    start: f64,

    /// End frequency in MHz
    #[arg(long, default_value_t = 108.0)]
    end: f64,

    /// Step size in kHz
    #[arg(long, default_value_t = 100.0)]
    step: f64,

    /// Sample rate in Hz
    #[arg(long, default_value_t = 2_048_000)]
    sample_rate: u32,

    /// Dwell time per frequency step in ms
    #[arg(long, default_value_t = 50)]
    dwell: u64,

    /// Signal detection threshold in dB above noise floor
    #[arg(long, default_value_t = 6.0)]
    threshold: f64,

    /// Tuner gain in dB (e.g., 40.0). Omit for auto gain.
    #[arg(short, long)]
    gain: Option<f64>,

    /// Headless mode: log detected signals to stdout instead of TUI
    #[arg(long)]
    headless: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop))
        .expect("registering SIGINT handler");

    let config = parse_source_arg(&args.source, args.sample_rate)?;
    let mut source = sdr::open_source(config)?;
    source.set_sample_rate(args.sample_rate)?;

    // Use manual gain for scanning — AGC compresses dynamic range
    let gain = args.gain.unwrap_or(40.0);
    source.set_gain(Some((gain * 10.0) as i32))?;

    let start_hz = (args.start * 1e6) as u64;
    let end_hz = (args.end * 1e6) as u64;
    let step_hz = (args.step * 1e3) as u32;

    let mut scanner = FrequencyScanner::new(
        start_hz,
        end_hz,
        step_hz,
        args.sample_rate,
        args.threshold as f32,
    );

    let buf_size = (args.sample_rate as usize * args.dwell as usize / 1000).max(1024);
    let buf_size = buf_size.min(16384);
    let mut iq_buf = vec![Complex::new(0.0f32, 0.0); buf_size];

    if args.headless {
        run_headless(&mut source, &mut scanner, &mut iq_buf, &stop)
    } else {
        run_tui(&mut source, &mut scanner, &mut iq_buf, &stop)
    }
}

fn run_headless(
    source: &mut Box<dyn sdr::SdrSource>,
    scanner: &mut FrequencyScanner,
    iq_buf: &mut [Complex<f32>],
    stop: &Arc<AtomicBool>,
) -> Result<()> {
    let start_mhz = scanner.start_hz() as f64 / 1e6;
    let end_mhz = scanner.end_hz() as f64 / 1e6;
    println!(
        "Scanning {start_mhz:.1} - {end_mhz:.1} MHz | Gain: manual | Ctrl-C to stop"
    );
    println!();

    while !stop.load(Ordering::Relaxed) {
        let freq = scanner.current_freq();
        source
            .set_frequency(freq as u32)
            .with_context(|| format!("tuning to {} Hz", freq))?;

        let n = source.read(iq_buf).context("reading IQ samples")?;
        if n == 0 {
            scanner.reset();
            continue;
        }

        scanner.measure(&iq_buf[..n]);

        let sweep_complete = scanner.step();

        if sweep_complete {
            let signals = scanner.signals();
            let sweep = scanner.sweep_count();
            let floor = scanner.noise_floor();

            println!("── Sweep #{sweep} ── floor: {floor:.1} dB ──");

            if signals.is_empty() {
                println!("  (no signals detected)");
            } else {
                for s in signals {
                    let freq_mhz = s.freq_hz as f64 / 1e6;
                    let above_floor = s.power_db - floor;
                    println!(
                        "  {:>7.1} MHz  {:>6.1} dB  (+{:.1} dB)",
                        freq_mhz, s.power_db, above_floor
                    );
                }
            }
            println!();
        }
    }

    println!("Shutting down.");
    Ok(())
}

fn run_tui(
    source: &mut Box<dyn sdr::SdrSource>,
    scanner: &mut FrequencyScanner,
    iq_buf: &mut [Complex<f32>],
    stop: &Arc<AtomicBool>,
) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = (|| -> Result<()> {
        loop {
            if event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('r') => scanner.reset(),
                            _ => {}
                        }
                    }
                }
            }

            if stop.load(Ordering::Relaxed) {
                break;
            }

            let freq = scanner.current_freq();
            source
                .set_frequency(freq as u32)
                .with_context(|| format!("tuning to {} Hz", freq))?;

            let n = source.read(iq_buf).context("reading IQ samples")?;
            if n == 0 {
                scanner.reset();
                continue;
            }

            scanner.measure(&iq_buf[..n]);
            scanner.step();

            terminal.draw(|frame| {
                ui::render(frame, scanner);
            })?;
        }

        Ok(())
    })();

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}
