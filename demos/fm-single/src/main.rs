//! A complete FM broadcast receiver, in one file, with no project dependencies.
//!
//!     rtl_sdr -f 97700000 -s 960000 -g 40 - | fm-single
//!     rtl_sdr -f 97700000 -s 960000 -g 40 - | fm-single recording.wav
//!
//! Reads raw 8-bit IQ samples on stdin and plays audio on the default output.
//! Everything between those two points is below.

use std::collections::VecDeque;
use std::error::Error;
use std::f32::consts::PI;
use std::io::Read;
use std::sync::{Arc, Mutex};

// ===========================================================================
// The whole design, in three numbers.
//
// Each rate divides evenly into the one above it, so every decimation step is
// a whole number of samples. Pick rates that do not divide evenly and you get
// a slow drift between how fast audio is produced and how fast the sound card
// consumes it.
// ===========================================================================

const SDR_RATE: u32 = 960_000; // what rtl_sdr hands us
const IF_RATE: u32 = 240_000; // after step 1  (960 / 4)
const AUDIO_RATE: u32 = 48_000; // what the speakers want (240 / 5)

const IQ_DECIM: usize = (SDR_RATE / IF_RATE) as usize; // 4
const AUDIO_DECIM: usize = (IF_RATE / AUDIO_RATE) as usize; // 5

const CHANNEL_HALF_WIDTH: f32 = 100_000.0; // FM channel is ~200 kHz wide
const AUDIO_BANDWIDTH: f32 = 15_000.0; // mono FM audio tops out here
const MAX_DEVIATION: f32 = 75_000.0; // how far the carrier swings
const DEEMPHASIS_TAU: f32 = 75e-6; // 75 µs in North America, 50 µs in Europe
const VOLUME: f32 = 0.30;

/// One IQ sample: a point on the complex plane.
#[derive(Clone, Copy)]
struct Iq {
    i: f32,
    q: f32,
}

// ===========================================================================
// STEP 1 — Low-pass filter, and throw away samples we no longer need.
//
// The dongle hands us 960 kHz of spectrum. We want one 200 kHz station out of
// it. Filter away everything else, then keep only every 4th sample: once the
// high frequencies are gone, the extra samples carry no information.
// ===========================================================================

struct LowPass {
    taps: Vec<f32>,
    history: Vec<Iq>,
    pos: usize,
    decimation: usize,
    countdown: usize,
}

impl LowPass {
    /// `cutoff` is a fraction of the sample rate, from 0.0 to 0.5.
    fn new(cutoff: f32, num_taps: usize, decimation: usize) -> Self {
        Self {
            taps: design_lowpass(cutoff, num_taps),
            history: vec![Iq { i: 0.0, q: 0.0 }; num_taps],
            pos: 0,
            decimation,
            countdown: 0,
        }
    }

    fn process(&mut self, input: &[Iq]) -> Vec<Iq> {
        let mut out = Vec::with_capacity(input.len() / self.decimation + 1);
        let n = self.taps.len();

        for &sample in input {
            self.history[self.pos] = sample;
            self.pos = (self.pos + 1) % n;
            self.countdown += 1;

            // Only do the expensive part on the samples we are keeping.
            if self.countdown >= self.decimation {
                self.countdown = 0;
                let (mut i, mut q) = (0.0, 0.0);
                for j in 0..n {
                    let h = self.history[(self.pos + j) % n];
                    i += h.i * self.taps[j];
                    q += h.q * self.taps[j];
                }
                out.push(Iq { i, q });
            }
        }
        out
    }

    /// Audio is real-valued, so run it through as IQ with an empty Q.
    fn process_real(&mut self, input: &[f32]) -> Vec<f32> {
        let as_iq: Vec<Iq> = input.iter().map(|&x| Iq { i: x, q: 0.0 }).collect();
        self.process(&as_iq).iter().map(|s| s.i).collect()
    }
}

/// A Hamming-windowed sinc: the textbook low-pass filter.
///
/// An ideal low-pass is a sinc in the time domain, but it is infinitely long.
/// Chop it off and the sharp edges ring; multiply by a window first and they
/// do not. Normalising at the end makes the filter pass DC at unity gain.
fn design_lowpass(cutoff: f32, num_taps: usize) -> Vec<f32> {
    let center = (num_taps - 1) as f32 / 2.0;
    let mut taps: Vec<f32> = (0..num_taps)
        .map(|k| {
            let x = k as f32 - center;
            let sinc = if x.abs() < 1e-6 {
                2.0 * cutoff
            } else {
                (2.0 * PI * cutoff * x).sin() / (PI * x)
            };
            let hamming = 0.54 - 0.46 * (2.0 * PI * k as f32 / (num_taps - 1) as f32).cos();
            sinc * hamming
        })
        .collect();

    let sum: f32 = taps.iter().sum();
    if sum.abs() > 1e-10 {
        for t in &mut taps {
            *t /= sum;
        }
    }
    taps
}

// ===========================================================================
// STEP 2 — FM demodulation.
//
// FM encodes audio as the *speed of rotation* around the origin. So the audio
// is how far the point rotated between one sample and the next.
//
// Multiplying by the conjugate of the previous sample subtracts the previous
// angle from the current one. The angle of that product is the rotation, and
// the rotation is the audio. This is the whole of FM.
// ===========================================================================

struct FmDemod {
    prev: Iq,
    gain: f32,
}

impl FmDemod {
    fn new(sample_rate: f32) -> Self {
        Self {
            prev: Iq { i: 0.0, q: 0.0 },
            // Scale so a full-deviation swing lands near +/-1.0
            gain: sample_rate / (2.0 * PI * MAX_DEVIATION),
        }
    }

    fn process(&mut self, input: &[Iq]) -> Vec<f32> {
        input
            .iter()
            .map(|&s| {
                // s * conj(prev), written out rather than hidden in a library
                let re = s.i * self.prev.i + s.q * self.prev.q;
                let im = s.q * self.prev.i - s.i * self.prev.q;
                self.prev = s;
                im.atan2(re) * self.gain
            })
            .collect()
    }
}

// ===========================================================================
// STEP 3 — De-emphasis.
//
// FM stations boost their treble before transmitting, because hiss lives up
// there and boosting the signal first means it survives better. We undo that
// boost. Skip this and every station sounds harsh.
// ===========================================================================

struct DeEmphasis {
    alpha: f32,
    prev: f32,
}

impl DeEmphasis {
    fn new(tau: f32, sample_rate: f32) -> Self {
        Self {
            alpha: 1.0 - (-1.0 / (tau * sample_rate)).exp(),
            prev: 0.0,
        }
    }

    fn process(&mut self, samples: &mut [f32]) {
        for s in samples.iter_mut() {
            self.prev += self.alpha * (*s - self.prev);
            *s = self.prev;
        }
    }
}

// ===========================================================================
// Putting it together.
// ===========================================================================

fn main() -> Result<(), Box<dyn Error>> {
    let wav_path = std::env::args().nth(1);

    // Build the pipeline. Cutoffs are fractions of each stage's input rate.
    let mut iq_filter = LowPass::new(
        CHANNEL_HALF_WIDTH / SDR_RATE as f32,
        51,
        IQ_DECIM,
    );
    let mut fm_demod = FmDemod::new(IF_RATE as f32);
    let mut audio_filter = LowPass::new(
        AUDIO_BANDWIDTH / IF_RATE as f32,
        31,
        AUDIO_DECIM,
    );
    let mut deemphasis = DeEmphasis::new(DEEMPHASIS_TAU, AUDIO_RATE as f32);

    let ring = Arc::new(AudioRing::new(AUDIO_RATE as usize * 2));
    let _stream = start_audio(AUDIO_RATE, ring.clone())?;

    let mut wav = wav_path
        .map(|p| {
            hound::WavWriter::create(
                &p,
                hound::WavSpec {
                    channels: 1,
                    sample_rate: AUDIO_RATE,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                },
            )
        })
        .transpose()?;

    eprintln!("fm-single: {SDR_RATE} -> {IF_RATE} -> {AUDIO_RATE} Hz. Ctrl-C to stop.");

    // rtl_sdr writes interleaved unsigned bytes: I, Q, I, Q, ...
    // An even-sized buffer keeps us aligned to sample boundaries.
    let mut stdin = std::io::stdin().lock();
    let mut raw = vec![0u8; 65536];

    while stdin.read_exact(&mut raw).is_ok() {
        // Bytes are 0..255 with silence at 127.5; centre them on zero.
        let iq: Vec<Iq> = raw
            .chunks_exact(2)
            .map(|pair| Iq {
                i: (pair[0] as f32 - 127.5) / 127.5,
                q: (pair[1] as f32 - 127.5) / 127.5,
            })
            .collect();

        let tuned = iq_filter.process(&iq); // step 1
        let raw_audio = fm_demod.process(&tuned); // step 2
        let mut audio = audio_filter.process_real(&raw_audio); // step 1 again, on audio
        deemphasis.process(&mut audio); // step 3

        for s in &mut audio {
            *s = (*s * VOLUME).clamp(-1.0, 1.0);
        }

        if let Some(w) = wav.as_mut() {
            for &s in &audio {
                w.write_sample(s)?;
            }
        }
        ring.push(&audio);
    }

    if let Some(w) = wav {
        w.finalize()?;
    }
    Ok(())
}

// ===========================================================================
// Audio plumbing. Not radio — just getting samples to the sound card.
// ===========================================================================

/// Hands samples from the decode loop to the audio callback. If the sound card
/// falls behind we drop the oldest audio rather than block the radio.
struct AudioRing {
    buf: Mutex<VecDeque<f32>>,
    capacity: usize,
}

impl AudioRing {
    fn new(capacity: usize) -> Self {
        Self {
            buf: Mutex::new(VecDeque::with_capacity(capacity)),
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
        for s in out.iter_mut() {
            *s = buf.pop_front().unwrap_or(0.0);
        }
    }
}

fn start_audio(rate: u32, ring: Arc<AudioRing>) -> Result<cpal::Stream, Box<dyn Error>> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let device = cpal::default_host()
        .default_output_device()
        .ok_or("no audio output device")?;

    let stream = device.build_output_stream(
        &cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(rate),
            buffer_size: cpal::BufferSize::Default,
        },
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| ring.pop(data),
        |err| eprintln!("audio error: {err}"),
        None,
    )?;
    stream.play()?;
    Ok(stream)
}
