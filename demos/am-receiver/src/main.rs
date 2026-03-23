use anyhow::{Context, Result};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use num_complex::Complex;
use sdr::dsp::{am::AmDemodulator, filter::LowPassFilter};
use sdr::source::parse_source_arg;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Parser)]
#[command(name = "am-receiver", about = "AM radio receiver using RTL-SDR")]
struct Args {
    /// SDR source: file:PATH, tcp:HOST:PORT, or usb[:INDEX]
    #[arg(short, long, default_value = "tcp:127.0.0.1:1234")]
    source: String,

    /// AM frequency in MHz (e.g., 7.85 for CHU, 121.5 for aviation)
    #[arg(short, long, default_value_t = 7.85)]
    frequency: f64,

    /// RTL-SDR sample rate in Hz
    #[arg(long, default_value_t = 1_024_000)]
    sample_rate: u32,

    /// Audio output sample rate in Hz
    #[arg(long, default_value_t = 48_000)]
    audio_rate: u32,

    /// Tuner gain in dB (e.g., 49.0). Omit for auto gain.
    #[arg(short, long)]
    gain: Option<f64>,

    /// Write audio to a WAV file
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop))
        .expect("registering SIGINT handler");

    let freq_hz = (args.frequency * 1e6) as u32;
    let sdr_rate = args.sample_rate;
    let audio_rate = args.audio_rate;

    println!("AM Receiver");
    println!("  Frequency: {:.3} MHz", args.frequency);
    println!("  Sample rate: {} Hz", sdr_rate);
    println!("  Audio rate: {} Hz", audio_rate);
    println!("  Source: {}", args.source);

    let config = parse_source_arg(&args.source, sdr_rate)?;
    let mut source = sdr::open_source(config)?;
    source.set_frequency(freq_hz)?;
    source.set_sample_rate(sdr_rate)?;

    if let Some(gain) = args.gain {
        source.set_gain(Some((gain * 10.0) as i32))?;
        println!("  Gain: {gain:.1} dB");
    }

    // DSP chain:
    // 1. Low-pass filter IQ to AM channel bandwidth (~10 kHz) and decimate.
    //    Aviation AM channels are 25 kHz wide, so 10 kHz cutoff is fine.
    let intermediate_rate = 128_000u32;
    let iq_cutoff = 12_500.0 / sdr_rate as f32; // half of 25 kHz AM channel
    let iq_decimation = (sdr_rate / intermediate_rate) as usize; // 8x
    let mut iq_filter = LowPassFilter::new(iq_cutoff, 51, iq_decimation);

    // 2. AM demodulate (envelope detection)
    let am_demod = AmDemodulator::new();

    // 3. Low-pass filter audio to ~5 kHz and decimate to audio rate
    let audio_cutoff = 6_000.0 / intermediate_rate as f32;
    let audio_decimation = (intermediate_rate / audio_rate) as usize;
    let mut audio_filter = LowPassFilter::new(audio_cutoff, 31, audio_decimation.max(1));

    // Ring buffer for audio output
    let ring_size = audio_rate as usize * 2;
    let ring = Arc::new(RingBuffer::new(ring_size));
    let ring_producer = ring.clone();

    // Start audio output
    let audio_stream = start_audio_output(audio_rate, ring)?;
    audio_stream.play().context("starting audio stream")?;

    // Optional WAV output
    let mut wav_writer = if let Some(ref path) = args.output {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: audio_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let writer = hound::WavWriter::create(path, spec)
            .with_context(|| format!("creating WAV file: {path}"))?;
        println!("  Writing to: {path}");
        Some(writer)
    } else {
        None
    };

    println!();
    println!("Receiving... Press Ctrl-C to stop.");

    let buf_size = 16384;
    let mut iq_buf = vec![Complex::new(0.0f32, 0.0); buf_size];

    while !stop.load(Ordering::Relaxed) {
        let n = source.read(&mut iq_buf)?;
        if n == 0 {
            println!("End of input.");
            break;
        }

        // Filter IQ and decimate
        let filtered_iq = iq_filter.process(&iq_buf[..n]);

        // AM demodulate — just take the magnitude, AC-coupled
        let audio_raw = am_demod.process_ac_coupled(&filtered_iq);

        // Filter and decimate to audio rate
        let mut audio = audio_filter.process_real(&audio_raw);

        // Scale to comfortable level
        for s in &mut audio {
            *s *= 8.0;
            *s = s.clamp(-1.0, 1.0);
        }

        // Write to WAV if requested
        if let Some(ref mut writer) = wav_writer {
            for &s in &audio {
                writer.write_sample(s)?;
            }
        }

        // Push to ring buffer
        ring_producer.push(&audio);
    }

    if let Some(writer) = wav_writer {
        writer.finalize()?;
        println!("WAV file written.");
    }

    println!("Shutting down.");
    drop(audio_stream);
    Ok(())
}

/// Lock-free single-producer single-consumer ring buffer for audio.
struct RingBuffer {
    buf: std::sync::Mutex<std::collections::VecDeque<f32>>,
    capacity: usize,
}

impl RingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buf: std::sync::Mutex::new(std::collections::VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    fn push(&self, samples: &[f32]) {
        let mut buf = self.buf.lock().unwrap();
        for &s in samples {
            if buf.len() >= self.capacity {
                buf.pop_front();
            }
            buf.push_back(s);
        }
    }

    fn pop(&self, out: &mut [f32]) {
        let mut buf = self.buf.lock().unwrap();
        for sample in out.iter_mut() {
            *sample = buf.pop_front().unwrap_or(0.0);
        }
    }
}

fn start_audio_output(
    sample_rate: u32,
    ring: Arc<RingBuffer>,
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

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            ring.pop(data);
        },
        |err| eprintln!("Audio output error: {err}"),
        None,
    )?;

    Ok(stream)
}
