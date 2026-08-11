use anyhow::{Context, Result, bail};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use num_complex::Complex;
use sdr::dsp::{deemphasis::DeEmphasisFilter, filter::LowPassFilter, fm::FmDemodulator};
use sdr::source::parse_source_arg;
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

    /// RTL-SDR sample rate in Hz. Must be a whole multiple of --audio-rate.
    #[arg(long, default_value_t = 960_000)]
    sample_rate: u32,

    /// Audio output sample rate in Hz
    #[arg(long, default_value_t = 48_000)]
    audio_rate: u32,

    /// Write audio to a WAV file instead of (in addition to) speakers
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
    // 1. Low-pass filter IQ to FM channel bandwidth (~200 kHz) and
    //    decimate to an intermediate rate. This removes adjacent channels
    //    and noise before FM demodulation.
    let plan = plan_rates(sdr_rate, audio_rate)?;
    let intermediate_rate = plan.intermediate_rate;
    println!(
        "  Chain: {} -> {} -> {} Hz (decimate {}x then {}x)",
        sdr_rate, intermediate_rate, audio_rate, plan.iq_decimation, plan.audio_decimation
    );

    let iq_cutoff = 100_000.0 / sdr_rate as f32;
    let mut iq_filter = LowPassFilter::new(iq_cutoff, 51, plan.iq_decimation);

    // 2. FM demodulate at the intermediate rate
    let mut fm_demod = FmDemodulator::new(75_000.0, intermediate_rate as f32);

    // 3. Low-pass filter audio to ~15 kHz and decimate to audio rate
    let audio_cutoff = 15_000.0 / intermediate_rate as f32;
    let mut audio_filter = LowPassFilter::new(audio_cutoff, 31, plan.audio_decimation);

    // 3. De-emphasis (75 μs for North America)
    let mut deemphasis = DeEmphasisFilter::new(75e-6, audio_rate as f32);

    // Ring buffer for audio output — large enough to absorb timing jitter
    let ring_size = audio_rate as usize * 2; // 2 seconds of buffer
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

    // Main processing loop — smaller reads for smoother flow
    let buf_size = 16384;
    let mut iq_buf = vec![Complex::new(0.0f32, 0.0); buf_size];

    while !stop.load(Ordering::Relaxed) {
        let n = source.read(&mut iq_buf)?;
        if n == 0 {
            println!("End of input.");
            break;
        }

        // Filter IQ and decimate to intermediate rate
        let filtered_iq = iq_filter.process(&iq_buf[..n]);

        // FM demodulate
        let audio_raw = fm_demod.process(&filtered_iq);

        // Filter and decimate to audio rate
        let mut audio = audio_filter.process_real(&audio_raw);

        // De-emphasis
        deemphasis.process(&mut audio);

        // Scale output to a comfortable listening level.
        // The FM demod gain produces values well outside [-1, 1] for
        // strong stations. Apply a volume reduction and soft clamp.
        for s in &mut audio {
            *s *= 0.30;
            *s = s.clamp(-1.0, 1.0);
        }

        // Write to WAV if requested
        if let Some(ref mut writer) = wav_writer {
            for &s in &audio {
                writer.write_sample(s)?;
            }
        }

        // Push to ring buffer (never blocks)
        ring_producer.push(&audio);
    }

    // Finalize WAV file
    if let Some(writer) = wav_writer {
        writer.finalize()?;
        println!("WAV file written.");
    }

    println!("Shutting down.");
    drop(audio_stream);
    Ok(())
}

/// The FM channel is ~200 kHz wide, so demodulation has to happen at or above
/// that rate. Below it the channel no longer fits and the audio comes apart.
const MIN_INTERMEDIATE_RATE: u32 = 200_000;

/// How to get from the SDR's rate down to the audio rate in two whole steps.
#[derive(Debug)]
struct RatePlan {
    iq_decimation: usize,
    intermediate_rate: u32,
    audio_decimation: usize,
}

/// Split the total decimation into two stages that each divide evenly.
///
/// Both stages keep every Nth sample, so N has to be a whole number. Round it
/// off instead and audio comes out at a rate other than the one we label it
/// with — the station plays at the wrong speed and pitch, with nothing in the
/// output to say so. That is why an SDR rate that is not a whole multiple of
/// the audio rate is rejected here rather than quietly truncated.
fn plan_rates(sdr_rate: u32, audio_rate: u32) -> Result<RatePlan> {
    if audio_rate == 0 || sdr_rate == 0 {
        bail!("sample rate and audio rate must both be non-zero");
    }
    if sdr_rate % audio_rate != 0 {
        bail!(
            "sample rate {sdr_rate} Hz is not a whole multiple of audio rate {audio_rate} Hz \
             ({sdr_rate}/{audio_rate} = {:.3}). Every decimation step keeps every Nth sample, \
             so a fractional ratio cannot be represented and the audio would play at the wrong \
             speed. Try --sample-rate 960000, 1920000, or 2400000 with --audio-rate 48000.",
            sdr_rate as f64 / audio_rate as f64
        );
    }

    let total = sdr_rate / audio_rate;

    // Decimate as hard as possible in the first stage — that is the expensive
    // filter — while leaving the intermediate rate wide enough for the channel.
    let iq_decimation = (1..=total)
        .filter(|d| total % d == 0 && sdr_rate / d >= MIN_INTERMEDIATE_RATE)
        .max()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "sample rate {sdr_rate} Hz cannot be split into stages that keep the \
                 intermediate rate at or above {MIN_INTERMEDIATE_RATE} Hz for a {audio_rate} Hz \
                 output. Use a higher --sample-rate."
            )
        })?;

    Ok(RatePlan {
        iq_decimation: iq_decimation as usize,
        intermediate_rate: sdr_rate / iq_decimation,
        audio_decimation: (total / iq_decimation) as usize,
    })
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
                buf.pop_front(); // drop oldest if full
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The property that matters: the two stages multiplied together must land
    /// exactly on the audio rate, with the demodulator's rate wide enough for
    /// the channel.
    fn assert_exact(sdr_rate: u32, audio_rate: u32) {
        let p = plan_rates(sdr_rate, audio_rate).expect("should plan");
        assert_eq!(
            sdr_rate / p.iq_decimation as u32,
            p.intermediate_rate,
            "stage 1 must divide evenly"
        );
        assert_eq!(
            p.intermediate_rate / p.audio_decimation as u32,
            audio_rate,
            "stage 2 must land on the audio rate"
        );
        assert_eq!(
            p.iq_decimation * p.audio_decimation,
            (sdr_rate / audio_rate) as usize,
            "stages must multiply to the total decimation"
        );
        assert!(
            p.intermediate_rate >= MIN_INTERMEDIATE_RATE,
            "intermediate rate {} too narrow for a 200 kHz channel",
            p.intermediate_rate
        );
    }

    #[test]
    fn default_rate_is_exact() {
        assert_exact(960_000, 48_000);
        let p = plan_rates(960_000, 48_000).unwrap();
        assert_eq!(p.iq_decimation, 4);
        assert_eq!(p.intermediate_rate, 240_000);
        assert_eq!(p.audio_decimation, 5);
    }

    #[test]
    fn other_usable_rates_are_exact() {
        for rate in [960_000, 1_920_000, 2_400_000, 1_440_000] {
            assert_exact(rate, 48_000);
        }
    }

    /// 2048000/48000 = 128/3. The old code truncated this to 40x and produced
    /// 51200 Hz audio labelled 48000 Hz; now it is refused outright.
    #[test]
    fn indivisible_rate_is_rejected_not_rounded() {
        let err = plan_rates(2_048_000, 48_000).unwrap_err().to_string();
        assert!(err.contains("not a whole multiple"), "unexpected: {err}");
    }

    #[test]
    fn rate_too_low_for_the_channel_is_rejected() {
        assert!(plan_rates(96_000, 48_000).is_err());
    }

    #[test]
    fn zero_rates_do_not_divide_by_zero() {
        assert!(plan_rates(960_000, 0).is_err());
        assert!(plan_rates(0, 48_000).is_err());
    }
}
