use anyhow::{Context, Result};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use num_complex::Complex;
use sdr::dsp::{deemphasis::DeEmphasisFilter, filter::LowPassFilter, fm::FmDemodulator};
use sdr::source::parse_source_arg;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Parser)]
#[command(name = "fm-receiver", about = "FM radio receiver using RTL-SDR")]
struct Args {
    /// SDR source: file:PATH, tcp:HOST:PORT, or usb[:INDEX]
    #[arg(short, long, default_value = "tcp:127.0.0.1:1234")]
    source: String,

    /// FM station frequency in MHz (e.g., 100.3)
    #[arg(short, long, default_value_t = 100.3)]
    frequency: f64,

    /// RTL-SDR sample rate in Hz
    #[arg(long, default_value_t = 2_048_000)]
    sample_rate: u32,

    /// Audio output sample rate in Hz
    #[arg(long, default_value_t = 48_000)]
    audio_rate: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop))
        .expect("registering SIGINT handler");

    let freq_hz = (args.frequency * 1e6) as u32;
    let sdr_rate = args.sample_rate;
    let audio_rate = args.audio_rate;

    println!("FM Receiver");
    println!("  Frequency: {:.1} MHz", args.frequency);
    println!("  Sample rate: {} Hz", sdr_rate);
    println!("  Audio rate: {} Hz", audio_rate);
    println!("  Source: {}", args.source);
    println!();

    // Open SDR source
    let config = parse_source_arg(&args.source, sdr_rate)?;
    let mut source = sdr::open_source(config)?;
    source.set_frequency(freq_hz)?;
    source.set_sample_rate(sdr_rate)?;

    // DSP chain:
    // 1. Low-pass filter the IQ signal to the FM channel bandwidth (~200 kHz)
    //    cutoff = 100kHz / sdr_rate = fraction of sample rate
    let fm_cutoff = 100_000.0 / sdr_rate as f32;
    let mut iq_filter = LowPassFilter::new(fm_cutoff, 51, 1);

    // 2. FM demodulate (75 kHz deviation for WBFM)
    let mut fm_demod = FmDemodulator::new(75_000.0, sdr_rate as f32);

    // 3. Low-pass filter audio to ~15 kHz and decimate to audio rate
    let audio_cutoff = 15_000.0 / sdr_rate as f32;
    let decimation = sdr_rate / audio_rate;
    let mut audio_filter = LowPassFilter::new(audio_cutoff, 101, decimation as usize);

    // 4. De-emphasis (75 μs for North America)
    let mut deemphasis = DeEmphasisFilter::new(75e-6, audio_rate as f32);

    // Audio output channel
    let (audio_tx, audio_rx) = mpsc::sync_channel::<Vec<f32>>(8);

    // Start audio output
    let audio_stream = start_audio_output(audio_rate, audio_rx)?;
    audio_stream.play().context("starting audio stream")?;

    println!("Receiving... Press Ctrl-C to stop.");

    // Main processing loop
    let buf_size = 65536;
    let mut iq_buf = vec![Complex::new(0.0f32, 0.0); buf_size];

    while !stop.load(Ordering::Relaxed) {
        let n = source.read(&mut iq_buf)?;
        if n == 0 {
            println!("End of input.");
            break;
        }
        let iq = &iq_buf[..n];

        // Filter IQ
        let filtered_iq = iq_filter.process(iq);

        // FM demodulate
        let audio_raw = fm_demod.process(&filtered_iq);

        // Filter and decimate to audio rate
        let mut audio = audio_filter.process_real(&audio_raw);

        // De-emphasis
        deemphasis.process(&mut audio);

        // Send to audio output (drop if buffer full — avoids blocking)
        let _ = audio_tx.try_send(audio);
    }

    println!("Shutting down.");
    drop(audio_stream);
    Ok(())
}

fn start_audio_output(
    sample_rate: u32,
    rx: mpsc::Receiver<Vec<f32>>,
) -> Result<cpal::Stream> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("no audio output device")?;

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let mut pending: Vec<f32> = Vec::new();
    let mut pos = 0;

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                // Refill pending buffer if needed
                if pos >= pending.len() {
                    match rx.try_recv() {
                        Ok(buf) => {
                            pending = buf;
                            pos = 0;
                        }
                        Err(_) => {
                            *sample = 0.0;
                            continue;
                        }
                    }
                }
                *sample = pending[pos];
                pos += 1;
            }
        },
        |err| eprintln!("Audio output error: {err}"),
        None,
    )?;

    Ok(stream)
}


