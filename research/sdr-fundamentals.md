# SDR Fundamentals and FM Demodulation

Technical reference for "Listening to the Radio with Rust." Covers the signal processing
theory behind the three demos: FM receiver, CHU decoder, and frequency scanner.

---

## Table of Contents

1. [IQ Sampling](#iq-sampling)
2. [FM Demodulation](#fm-demodulation)
3. [Digital Filtering](#digital-filtering)
4. [Signal Power and Spectrum Analysis](#signal-power-and-spectrum-analysis)

---

## IQ Sampling

### What Are IQ Samples?

An IQ sample is a pair of numbers: an **In-phase (I)** component and a **Quadrature (Q)**
component. Together they form a **complex number**:

```
s[n] = I[n] + j * Q[n]
```

where `j` is the imaginary unit (engineers use `j` instead of `i` to avoid confusion with
current).

Each sample encodes two properties of the signal at that instant:

- **Amplitude**: `A = sqrt(I^2 + Q^2)` -- the magnitude of the complex number
- **Phase**: `phi = atan2(Q, I)` -- the angle of the complex number

Think of each sample as a point on the complex plane, or equivalently, a vector from the
origin with a length (amplitude) and angle (phase). As time progresses, the point traces a
path on the complex plane. A pure tone at the center frequency appears as a point rotating
in a circle. A signal slightly above center frequency rotates counterclockwise; slightly
below rotates clockwise.

### Why Complex Numbers?

A real-valued signal (a single number per sample, like a microphone recording) can only
capture amplitude at each moment. It cannot distinguish between a frequency *above* the
center and the *same distance below* it -- these are "positive" and "negative" frequency
offsets, and they look identical in a real signal.

Complex-valued IQ samples solve this. By capturing both the cosine-aligned (I) and
sine-aligned (Q) components, they preserve the full instantaneous state of the signal:

- **Amplitude AND phase** at every sample
- **Positive and negative frequencies** are distinguishable
- The usable bandwidth equals the **full sample rate**, not half of it

This is the critical insight: with real sampling, the Nyquist theorem says you can capture
bandwidth up to `f_s / 2`. With complex (IQ) sampling, the usable bandwidth is `f_s`
because positive and negative frequencies are distinct.

```
Real sampling:    bandwidth = f_s / 2
Complex sampling: bandwidth = f_s
```

So at 2.4 MS/s (mega samples per second), an IQ-sampled SDR captures 2.4 MHz of spectrum
centered on the tuned frequency.

### How the RTL-SDR Produces IQ Data

The RTL-SDR Blog V4 contains two main chips:

1. **R820T2 (or R860) tuner**: An analog RF front-end that takes the antenna signal,
   amplifies it, and **downconverts** it to a low intermediate frequency (IF). It
   shifts the signal of interest from its original RF frequency down to baseband
   (centered around 0 Hz). This is done by mixing the incoming RF signal with a local
   oscillator (LO) -- the same principle as a superheterodyne receiver, just implemented
   on a chip.

2. **RTL2832U demodulator/ADC**: Originally designed for DVB-T television, this chip
   contains an 8-bit ADC and a digital downconverter. It samples the IF signal from the
   tuner at 28.8 MS/s using a sigma-delta ADC, then digitally resamples it down to
   whatever sample rate the user requests (up to 3.2 MS/s, though 2.56 MS/s is the
   practical maximum to avoid dropped samples). The output is a stream of 8-bit I and Q
   values sent over USB.

The quadrature (I/Q) separation happens inside the RTL2832U. It multiplies the digitized
signal by a cosine (producing I) and a sine (producing Q) at the IF frequency, then
low-pass filters both to produce the baseband IQ stream.

```
Signal path:

  Antenna
    |
    v
  [R820T2 Tuner]  -- RF amplification, filtering, downconversion to IF
    |
    v
  [RTL2832U]       -- ADC (28.8 MS/s sigma-delta) -> digital downconvert -> decimate
    |
    v
  USB bulk transfer -- interleaved 8-bit unsigned I, Q, I, Q, ...
    |
    v
  Your Rust code
```

### RTL-SDR Data Format

The RTL-SDR outputs IQ data as **interleaved unsigned 8-bit integers**:

```
Byte:   [I0] [Q0] [I1] [Q1] [I2] [Q2] ...
Value:   0-255 per byte
```

Each byte is an **unsigned** value in the range [0, 255]. To convert to a usable
signed representation centered on zero:

```
I_float = (I_raw as f32 - 127.5) / 127.5
Q_float = (Q_raw as f32 - 127.5) / 127.5
```

This maps unsigned [0, 255] to approximately [-1.0, +1.0]. Some implementations use
127 or 128 as the offset; 127.5 is most accurate since 8-bit unsigned has no true
center value. The choice matters very little in practice.

In Rust, reading from the RTL-SDR device produces a `&[u8]` buffer. Pairs of bytes
become complex samples:

```rust
// Convert raw RTL-SDR bytes to complex IQ samples
fn bytes_to_iq(buf: &[u8]) -> Vec<Complex<f32>> {
    buf.chunks_exact(2)
        .map(|pair| {
            let i = (pair[0] as f32 - 127.5) / 127.5;
            let q = (pair[1] as f32 - 127.5) / 127.5;
            Complex::new(i, q)
        })
        .collect()
}
```

At a sample rate of 2.4 MS/s, this produces **2.4 million complex samples per second**,
or **4.8 million bytes per second** (4.8 MB/s) over USB. This is why performance matters
and why Rust is a natural fit -- you need to process millions of samples per second in
real time without GC pauses or allocation overhead.

### Nyquist Theorem in Practice

The Nyquist-Shannon sampling theorem states:

> A signal can be perfectly reconstructed from its samples if the sampling rate is at
> least twice the bandwidth of the signal.

For **real** signals: `f_s >= 2 * f_max`

For **complex** (IQ) signals, the situation is different. Because complex samples
distinguish positive and negative frequencies, the usable bandwidth equals the full
sample rate:

```
Usable bandwidth = f_s (complex sampling)
```

So an RTL-SDR set to 2.4 MS/s can observe 2.4 MHz of spectrum, centered on the tuned
frequency. If you tune to 100.0 MHz, you see everything from 98.8 MHz to 101.2 MHz.

**Practical implications for the demos:**

| Demo           | Signal Bandwidth | Minimum Sample Rate | Practical Sample Rate |
|----------------|-----------------|--------------------|-----------------------|
| FM Broadcast   | ~200 kHz        | 200 kHz            | 1.0 - 2.4 MS/s       |
| CHU Time Signal| ~10 kHz (AM/SSB)| 10 kHz             | 240 kHz - 1.0 MS/s   |
| Freq Scanner   | Full sweep      | Varies per step    | 2.0 - 2.4 MS/s       |

Higher sample rates than the minimum are common because:
1. They provide a wider view of the spectrum (useful for the scanner)
2. Oversampling improves effective resolution beyond 8 bits
3. You can decimate later with better filtering

### Data Rates and Buffer Sizes

At 2.4 MS/s with 8-bit IQ:

```
Bytes per second:    2.4M samples * 2 bytes/sample = 4.8 MB/s
Bytes per millisecond: 4,800 bytes
Typical buffer size: 16,384 bytes (16 KB) = ~3.4 ms of data = ~8,192 complex samples
```

The RTL-SDR library delivers data in buffer callbacks. Each callback provides a chunk
of raw bytes (commonly 16 KB or 64 KB). Your processing pipeline must consume each
buffer before the next one arrives, or you will drop samples.

---

## FM Demodulation

### How FM Encoding Works

Frequency Modulation (FM) encodes an audio signal as **variations in the carrier's
instantaneous frequency**. When the audio signal goes positive, the carrier frequency
increases; when negative, it decreases. The carrier amplitude stays constant.

Mathematically, an FM signal is:

```
x(t) = A * cos(2*pi*f_c*t + 2*pi*k_f * integral(m(tau) d_tau, 0, t))
```

where:
- `f_c` is the carrier frequency
- `k_f` is the frequency sensitivity (Hz per unit of modulating signal)
- `m(t)` is the modulating (audio) signal
- The integral of the audio signal becomes the **phase** of the carrier

The **instantaneous frequency** is:

```
f_inst(t) = f_c + k_f * m(t)
```

So the audio signal `m(t)` is literally the deviation of the instantaneous frequency
from the carrier. To demodulate FM, you need to extract the instantaneous frequency.

### The Phase Difference Method

After IQ sampling, the signal is at baseband (centered on 0 Hz). Each complex sample
has a phase:

```
phi[n] = atan2(Q[n], I[n])
```

The instantaneous frequency is the rate of change of phase. In discrete time, that is
the **phase difference** between consecutive samples:

```
delta_phi[n] = phi[n] - phi[n-1]
```

The demodulated audio is proportional to this phase difference. However, computing
`atan2` on each sample and then subtracting has a critical problem: **phase wrapping**.
If the phase crosses the +/- pi boundary, the difference will spike by 2*pi, producing
a loud click in the audio. You would need to unwrap the phase, which adds complexity.

### The Conjugate Multiply Method (Preferred)

A better approach avoids `atan2` entirely by using the **conjugate multiply** trick.
Multiply the current sample by the **complex conjugate** of the previous sample:

```
y[n] = x[n] * conj(x[n-1])
```

where `conj(a + jb) = a - jb`.

Expanding this with `x[n] = I[n] + j*Q[n]`:

```
y[n] = (I[n] + j*Q[n]) * (I[n-1] - j*Q[n-1])
     = (I[n]*I[n-1] + Q[n]*Q[n-1]) + j*(Q[n]*I[n-1] - I[n]*Q[n-1])
```

The **phase of the result** `y[n]` equals the phase difference between consecutive
samples, with no wrapping issues:

```
angle(y[n]) = phi[n] - phi[n-1] = delta_phi
```

Then:

```
audio[n] = atan2(Im(y[n]), Re(y[n]))
         = atan2(Q[n]*I[n-1] - I[n]*Q[n-1], I[n]*I[n-1] + Q[n]*Q[n-1])
```

This is the formula from the talk outline. It computes the phase difference directly
without ever computing or unwrapping the absolute phase.

**Why this works**: Multiplying two complex numbers adds their angles. Conjugating
negates the angle. So `x[n] * conj(x[n-1])` has angle `phi[n] + (-phi[n-1])` =
`phi[n] - phi[n-1]`, which is exactly the phase difference.

### Approximation for High Sample Rates

When the sample rate is much higher than the signal bandwidth (which it usually is in
SDR), the phase change between consecutive samples is small. For small angles:

```
sin(theta) ~= theta
```

This means the **imaginary part** of the conjugate product is approximately equal to
the phase difference:

```
audio[n] ~= Im(y[n]) = Q[n]*I[n-1] - I[n]*Q[n-1]
```

This avoids the `atan2` call entirely, which is significant for performance. The
approximation holds well when the SDR sample rate is 6-10x the signal bandwidth. For
an FM broadcast signal (~200 kHz bandwidth) sampled at 2.4 MS/s, the oversampling ratio
is 12x, so this approximation is excellent.

### Implementation in Rust

```rust
use num_complex::Complex;

/// FM demodulate a stream of IQ samples using the conjugate multiply method.
/// Returns one audio sample per IQ sample.
fn fm_demodulate(samples: &[Complex<f32>]) -> Vec<f32> {
    samples
        .windows(2)
        .map(|w| {
            let product = w[1] * w[0].conj();
            product.arg() // atan2(im, re) -- exact method
            // or: product.im  // approximate method (faster, good at high sample rates)
        })
        .collect()
}
```

### Frequency Deviation

The maximum frequency deviation determines how far the carrier swings from its center
frequency. This is the key parameter distinguishing wideband and narrowband FM.

**Wideband FM (WFM)** -- FM broadcast radio:
- Peak deviation: +/- 75 kHz
- Maximum modulating frequency: 15 kHz (mono), 53 kHz (stereo with subcarriers)
- Bandwidth (Carson's rule): `BW = 2 * (75 + 15) = 180 kHz` (mono)
- Bandwidth with stereo/RDS: ~200 kHz
- Channel spacing: 200 kHz (in North America)
- Modulation index: `beta = 75 / 15 = 5` (wideband, beta > 1)

**Narrowband FM (NFM)** -- ham radio, public safety:
- Peak deviation: +/- 5 kHz (or +/- 2.5 kHz for "narrow" channels)
- Maximum modulating frequency: 3 kHz (voice)
- Bandwidth (Carson's rule): `BW = 2 * (5 + 3) = 16 kHz`
- Channel spacing: 12.5 - 25 kHz
- Modulation index: `beta = 5 / 3 ~= 1.7` (narrowband in FM broadcasting terms)

**Carson's Rule** gives an estimate of occupied bandwidth containing ~98% of signal power:

```
BW = 2 * (delta_f + f_m)
```

where `delta_f` is peak deviation and `f_m` is maximum modulating frequency.

**Implications for the demos:**
- **FM receiver**: Set a wide IF filter (~200 kHz), use high sample rate, decimate aggressively to audio
- **CHU decoder**: CHU uses AM (with some FM characteristics); narrowband filters apply
- **Freq scanner**: The scanner needs to know channel bandwidths to compute per-channel power

### De-emphasis Filter

FM broadcasting uses a **pre-emphasis / de-emphasis** system to reduce noise. High
frequencies are boosted before transmission (pre-emphasis) and attenuated in the receiver
(de-emphasis), which reduces the perceived high-frequency noise inherent in FM.

The de-emphasis filter is a simple single-pole low-pass filter defined by a time constant:

- **North America and South Korea**: tau = 75 microseconds, corner frequency = 2,122 Hz
- **Europe, Australia, Japan**: tau = 50 microseconds, corner frequency = 3,183 Hz

The corner frequency is:

```
f_c = 1 / (2 * pi * tau)

For 75 us: f_c = 1 / (2 * pi * 75e-6) = 2,122 Hz
For 50 us: f_c = 1 / (2 * pi * 50e-6) = 3,183 Hz
```

Above this frequency, the response rolls off at -6 dB/octave (-20 dB/decade).

**Digital implementation** uses a single-pole IIR filter designed via the bilinear
transform:

1. Prewarp the analog cutoff frequency:

```
w_p = 2 * pi * f_c
w_pp = tan(w_p / (2 * f_s))
```

where `f_s` is the audio sample rate.

2. Compute IIR coefficients:

```
a1 = (w_pp - 1) / (w_pp + 1)
b0 = w_pp / (1 + w_pp)
b1 = b0
```

3. Apply the filter:

```
y[n] = b0 * x[n] + b1 * x[n-1] - a1 * y[n-1]
```

This is a first-order IIR filter -- extremely cheap to compute (one multiply-add per
sample). It should be applied after FM demodulation and decimation to audio rate.

### Stereo FM Basics

FM stereo uses a clever multiplexing scheme that is backward-compatible with mono
receivers. The baseband signal after FM demodulation contains:

```
Baseband spectrum layout:

0 Hz         15 kHz  19 kHz  23 kHz        38 kHz        53 kHz
|--- L+R ---|        |       |--- L-R (DSB-SC) ---|
             mono    pilot   stereo difference
                     tone    (suppressed carrier AM at 38 kHz)
```

- **0 - 15 kHz**: L+R (mono-compatible sum signal)
- **19 kHz**: Pilot tone at 9% modulation (always present for stereo broadcasts)
- **23 - 53 kHz**: L-R difference signal, modulated as double-sideband suppressed-carrier
  AM centered at 38 kHz (exactly 2x the pilot frequency)
- **57 kHz**: Optional RDS (Radio Data System) subcarrier for station metadata

**Decoding process:**

1. Detect the 19 kHz pilot tone (indicates stereo broadcast)
2. Double the pilot frequency to regenerate the 38 kHz carrier (phase-locked)
3. Multiply the L-R signal by the regenerated 38 kHz carrier to demodulate it
4. Low-pass filter to extract the L-R audio (0 - 15 kHz)
5. Combine: `L = (L+R) + (L-R)` and `R = (L+R) - (L-R)`

A mono receiver simply ignores everything above 15 kHz, hearing only L+R. This is
why stereo FM is backward-compatible.

**For the demos**: The FM receiver demo can start as mono-only (just low-pass filter the
demodulated signal at 15 kHz). Stereo decoding is a nice stretch goal but adds
significant complexity.

---

## Digital Filtering

### Why Filtering Matters in SDR

Filtering is the second most important operation in SDR after sampling itself. You need
filters for:

1. **Decimation**: Before downsampling, a low-pass filter prevents aliasing
2. **Channel selection**: A band-pass filter isolates one signal from neighbors
3. **Noise reduction**: Low-pass filters remove out-of-band noise
4. **De-emphasis**: The single-pole IIR filter described above
5. **Signal conditioning**: Shaping the signal before demodulation

### Low-Pass Filters for Decimation

After FM demodulation, the audio signal has a bandwidth of ~15 kHz but may be sampled
at 240 kHz or higher. Before downsampling to 48 kHz audio rate, a low-pass filter
must remove all energy above 24 kHz (the Nyquist limit at 48 kHz) to prevent aliasing.

**Decimation** = low-pass filter + downsample:

```
Input: 240,000 samples/sec (post-demod, still at intermediate rate)
Low-pass filter: cutoff at ~16 kHz, reject above 24 kHz
Downsample by 5: keep every 5th sample
Output: 48,000 samples/sec (audio rate)
```

The decimation factor from the SDR sample rate to audio rate can be large. For example,
from 2.4 MS/s to 48 kHz is a factor of 50. This is typically done in stages:

```
Stage 1: 2,400,000 -> 240,000 (decimate by 10)
         Low-pass at ~100 kHz to capture full FM bandwidth
         [FM demodulation happens here, on the 240 kHz stream]

Stage 2: 240,000 -> 48,000 (decimate by 5)
         Low-pass at ~16 kHz for audio bandwidth
```

Multi-stage decimation is more efficient than a single large decimation because each
stage's filter can have a more relaxed transition band, requiring fewer taps.

### Band-Pass Filters for Channel Selection

When the SDR captures 2.4 MHz of spectrum, there may be multiple FM stations or signals
in that bandwidth. A band-pass filter isolates one signal:

```
SDR captures: 98.8 MHz to 101.2 MHz (2.4 MHz wide, centered on 100.0 MHz)
Target station: 99.7 MHz (appears at -300 kHz offset in baseband)
Band-pass filter: centered at -300 kHz, width ~200 kHz
```

In practice, channel selection is often done by:
1. Frequency-shifting the desired signal to 0 Hz (multiply by a complex exponential)
2. Low-pass filtering to the channel bandwidth
3. Decimating

This is equivalent to a band-pass filter but easier to implement because low-pass
filter design is simpler than band-pass.

```rust
// Frequency shift: move a signal at offset_hz to 0 Hz
fn frequency_shift(
    samples: &[Complex<f32>],
    offset_hz: f32,
    sample_rate: f32,
) -> Vec<Complex<f32>> {
    samples
        .iter()
        .enumerate()
        .map(|(n, &s)| {
            let t = n as f32 / sample_rate;
            let shift = Complex::from_polar(1.0, -2.0 * PI * offset_hz * t);
            s * shift
        })
        .collect()
}
```

### FIR vs IIR Filters

**FIR (Finite Impulse Response)** filters compute output as a weighted sum of the
current and past N input values:

```
y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] + ... + bN*x[n-N]
```

Properties:
- **Always stable** (no feedback, so they cannot oscillate)
- **Linear phase** (all frequencies delayed by the same amount -- no phase distortion)
- **Higher order** needed for sharp cutoffs (more taps = more computation)
- **Efficient for decimation**: only compute output samples that will be kept
  (compute 1 out of every M samples when decimating by M)

**IIR (Infinite Impulse Response)** filters use feedback from previous output values:

```
y[n] = b0*x[n] + b1*x[n-1] + ... - a1*y[n-1] - a2*y[n-2] - ...
```

Properties:
- **Can be unstable** if coefficient precision is insufficient
- **Non-linear phase** (different frequencies are delayed differently)
- **Much sharper cutoff** for a given order (fewer coefficients needed)
- **Cannot skip output computation** during decimation (feedback depends on all outputs)
- **Very efficient for simple filters** like de-emphasis (1-2 coefficients)

**Which to use when:**

| Use Case                     | Recommended | Why                                           |
|-----------------------------|-------------|-----------------------------------------------|
| Decimation anti-alias filter | FIR         | Can skip output computation; linear phase      |
| Channel selection filter     | FIR         | Linear phase preserves signal; efficient decim |
| De-emphasis filter           | IIR         | Single pole, trivially cheap                   |
| Audio equalization           | IIR         | Sharp response with few coefficients           |
| Matched filter / correlation | FIR         | Requires specific impulse response shape       |

**For the demos**: Use FIR filters for the main signal processing pipeline (decimation,
channel selection) and a single-pole IIR for de-emphasis. This is the standard approach
in SDR software.

### FIR Filter Design

A common approach for SDR is the **windowed sinc** method:

1. Start with the ideal low-pass filter impulse response (a sinc function):

```
h_ideal[n] = sin(2*pi*f_c*n) / (pi*n)    for n != 0
h_ideal[0] = 2*f_c
```

where `f_c` is the normalized cutoff frequency (cutoff / sample_rate).

2. Apply a window function to make it finite and reduce spectral leakage:

```
h[n] = h_ideal[n] * w[n]
```

Common windows ranked by stopband attenuation:
- **Hamming**: -53 dB (good default for most SDR work)
- **Blackman**: -74 dB (better rejection, wider transition)
- **Kaiser**: configurable tradeoff (most flexible)

3. Normalize so the filter has unity gain at DC:

```
h_normalized[n] = h[n] / sum(h)
```

**Number of taps** (filter length) determines the sharpness of the transition band.
A rule of thumb for the Hamming window:

```
N ~= 4 / (transition_width / sample_rate)
```

For a transition from 16 kHz to 24 kHz at 240 kHz sample rate:

```
transition_width = (24000 - 16000) / 240000 = 0.033
N ~= 4 / 0.033 ~= 121 taps
```

### CIC Filters for Large Decimation Ratios

For the first stage of decimation where the ratio is large (e.g., 10x or more),
**Cascaded Integrator-Comb (CIC)** filters are very efficient:

- No multiplications needed (only additions and subtractions)
- Fixed response shape (sinc-like, with nulls at multiples of the output rate)
- Passband droop that must be compensated by a later FIR stage

A CIC decimator followed by a compensation FIR is the standard architecture in SDR
for taking high sample rates down to a manageable intermediate rate.

---

## Signal Power and Spectrum Analysis

### Computing Signal Power from IQ Samples

The instantaneous power of a complex IQ sample is the **magnitude squared**:

```
P[n] = |x[n]|^2 = I[n]^2 + Q[n]^2
```

Note: no square root needed. Magnitude squared is cheaper to compute and is directly
proportional to power.

**Average power** over a block of N samples:

```
P_avg = (1/N) * sum(I[n]^2 + Q[n]^2, n=0..N-1)
```

**Power in decibels** (relative to full scale for 8-bit samples):

```
P_dB = 10 * log10(P_avg)
```

In Rust:

```rust
fn average_power(samples: &[Complex<f32>]) -> f32 {
    let sum: f32 = samples.iter().map(|s| s.norm_sqr()).sum();
    sum / samples.len() as f32
}

fn power_db(power: f32) -> f32 {
    10.0 * power.log10()
}
```

### FFT for Frequency Domain Analysis

The **Fast Fourier Transform (FFT)** converts a block of time-domain IQ samples into
the frequency domain, showing the power at each frequency within the captured bandwidth.

Given N complex IQ samples, the FFT produces N complex frequency bins:

```
X[k] = sum(x[n] * e^(-j*2*pi*k*n/N), n=0..N-1)    for k = 0..N-1
```

Each bin `k` corresponds to a frequency:

```
f[k] = (k - N/2) * f_s / N     (after fftshift)
```

The frequency resolution (bin width) is:

```
delta_f = f_s / N
```

So with 2.4 MS/s and a 1024-point FFT:

```
Bin width = 2,400,000 / 1024 = 2,343.75 Hz
```

### Power Spectral Density (PSD)

To compute the PSD from IQ samples:

1. **Window** the samples to reduce spectral leakage (Hamming, Hann, Blackman, etc.):
   ```
   x_w[n] = x[n] * w[n]
   ```

2. **Compute FFT**:
   ```
   X[k] = FFT(x_w)
   ```

3. **Compute magnitude squared** (power):
   ```
   P[k] = |X[k]|^2 = Re(X[k])^2 + Im(X[k])^2
   ```

4. **Normalize** by FFT size and window energy:
   ```
   PSD[k] = P[k] / (N * sum(w[n]^2))
   ```

5. **Convert to dB**:
   ```
   PSD_dB[k] = 10 * log10(PSD[k])
   ```

6. **FFT shift**: Rearrange so that negative frequencies are on the left, DC in the
   center, positive frequencies on the right (swap the two halves of the array).

**Averaging**: For a cleaner spectrum display, compute the PSD over many overlapping
blocks and average. This reduces variance at the cost of time resolution.

### Windowing Functions

Without windowing, the sharp edges of a finite block of samples cause **spectral
leakage** -- energy from strong signals smears across the entire spectrum. Windows
taper the edges smoothly to zero, trading frequency resolution for reduced leakage.

Common choices for SDR spectrum analysis:

| Window   | Main Lobe Width | Sidelobe Level | Use Case                         |
|----------|----------------|----------------|----------------------------------|
| None     | Narrowest      | -13 dB         | Maximum resolution, poor dynamic |
| Hann     | Moderate       | -31 dB         | General purpose                  |
| Hamming  | Moderate       | -43 dB         | Good default for SDR             |
| Blackman | Wide           | -58 dB         | When dynamic range matters       |
| Flat-top | Very wide      | -93 dB         | Amplitude accuracy               |

For the frequency scanner demo, Hamming or Blackman are good choices -- you want
decent dynamic range to distinguish weak signals from the noise floor.

### How a Frequency Scanner Works

The frequency scanner demo sweeps across a range of frequencies, measuring signal
power at each step. Here is the algorithm:

```
1. Define sweep parameters:
   - Start frequency (e.g., 88.0 MHz)
   - Stop frequency (e.g., 108.0 MHz)
   - Step size (e.g., 100 kHz -- half the FM channel spacing)
   - Dwell time per step (e.g., 10 ms)

2. For each frequency step:
   a. Retune the SDR to the new center frequency
      (this takes ~5 ms on RTL-SDR due to PLL settling)
   b. Discard the first buffer of samples (stale data from previous frequency)
   c. Collect samples for the dwell time
   d. Compute average power: P = mean(I^2 + Q^2)
      -- or for finer resolution, compute FFT and measure peak/total power
         within the channel bandwidth
   e. Record: (frequency, power_dB)

3. After full sweep:
   a. Display frequency vs. power (bar chart, spectrum plot)
   b. Identify active channels: frequencies where power exceeds
      a threshold (e.g., noise floor + 10 dB)
   c. Optionally: tune to the strongest signal and demodulate
```

**Alternative: Wideband FFT approach**

Instead of retuning for each step, capture the full 2.4 MHz bandwidth at once and
compute the FFT. This gives you the spectrum of everything in that window
simultaneously. Then step the center frequency by 2.4 MHz and repeat. This is faster
because you make fewer retune calls.

```
Wideband sweep of 88-108 MHz with 2.4 MHz capture bandwidth:
- Step 1: Tune to 89.2 MHz, capture 88.0 - 90.4 MHz
- Step 2: Tune to 91.6 MHz, capture 90.4 - 92.8 MHz
- ...
- Step 9: Tune to 108.0 MHz, capture 106.8 - 109.2 MHz (trim edges)
- Stitch the spectra together
```

This requires only ~9 retune operations instead of 200, making the sweep much faster.

### Signal Power Measurement Approaches

For the scanner, there are two useful power measurements:

**Peak power**: Maximum FFT bin value across the channel bandwidth. Good for detecting
narrow signals (like a carrier or narrowband FM).

```rust
fn peak_power_db(fft_bins: &[f32], start_bin: usize, end_bin: usize) -> f32 {
    let peak = fft_bins[start_bin..end_bin]
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max);
    10.0 * peak.log10()
}
```

**Channel power**: Sum of all FFT bin values across the channel bandwidth. Better for
wideband signals (like FM broadcast) where energy is spread across many bins.

```rust
fn channel_power_db(fft_bins: &[f32], start_bin: usize, end_bin: usize) -> f32 {
    let total: f32 = fft_bins[start_bin..end_bin].iter().sum();
    10.0 * total.log10()
}
```

**DC spike removal**: RTL-SDR devices produce a spike at the center frequency (DC offset /
LO leakage). For power measurements, zero out the center few bins of the FFT to avoid
this artifact skewing results.

---

## Putting It All Together: The FM Receiver Pipeline

The complete signal processing chain for the FM receiver demo:

```
RTL-SDR (2.4 MS/s, 8-bit unsigned IQ)
  |
  v
Byte-to-complex conversion (subtract 127.5, normalize)
  |
  v
[Optional] Frequency shift to center desired station at 0 Hz
  |
  v
Low-pass filter + decimate to ~240 kHz (FIR, capture 200 kHz FM channel)
  |
  v
FM demodulate (conjugate multiply, take arg or imaginary part)
  |
  v
Low-pass filter + decimate to 48 kHz (FIR, cutoff at 15 kHz for mono)
  |
  v
De-emphasis filter (single-pole IIR, 75 us time constant)
  |
  v
Audio output (48 kHz, 16-bit PCM or f32 to sound card)
```

Each stage is a natural fit for Rust iterator adapters, and the entire pipeline can be
zero-copy with careful buffer management.

---

## References

- [IQ Sampling - PySDR](https://pysdr.org/content/sampling.html)
- [Frequency Domain - PySDR](https://pysdr.org/content/frequency_domain.html)
- [IQ Files and SigMF - PySDR](https://pysdr.org/content/iq_files.html)
- [I/Q Signals 101 - Wireless Pi](https://wirelesspi.com/i-q-signals-101-neither-complex-nor-complicated/)
- [FM and Demodulation Using DSP Techniques - Wireless Pi](https://wirelesspi.com/frequency-modulation-fm-and-demodulation-using-dsp-techniques/)
- [FIR vs IIR Filters - Wireless Pi](https://wirelesspi.com/fir-vs-iir-filters-a-practical-comparison/)
- [DSP Tricks: Frequency Demodulation Algorithms - Embedded.com](https://www.embedded.com/dsp-tricks-frequency-demodulation-algorithms/)
- [Understanding I/Q Signals and Quadrature Modulation - All About Circuits](https://www.allaboutcircuits.com/textbook/radio-frequency-analysis-design/radio-frequency-demodulation/understanding-i-q-signals-and-quadrature-modulation/)
- [FM Deemphasis - GNU Radio Wiki](https://wiki.gnuradio.org/index.php/FM_Deemphasis)
- [FM Frequency Modulation Index & Deviation Ratio - Electronics Notes](https://www.electronics-notes.com/articles/radio/modulation/fm-frequency-modulation-index-deviation-ratio.php)
- [Stereo VHF FM Broadcast - Electronics Notes](https://www.electronics-notes.com/articles/audio-video/broadcast-audio/vhf-fm-stereo.php)
- [Carson Bandwidth Rule - Wikipedia](https://en.wikipedia.org/wiki/Carson_bandwidth_rule)
- [Processing IQ Data Formats - K3XEC](https://k3xec.com/packrat-processing-iq/)
- [IQ Data Format Introduction - rtl_433](https://triq.org/rtl_433/IQ_FORMATS.html)
- [RTL-SDR Blog V4 Users Guide](https://www.rtl-sdr.com/v4/)
- [About RTL-SDR](https://www.rtl-sdr.com/about-rtl-sdr/)
- [Decimation FAQ - dspGuru](https://dspguru.com/dsp/faqs/multirate/decimation/)
- [A Quadrature Signals Tutorial - DSPRelated](https://www.dsprelated.com/showarticle/192.php)
- [Difference Between IIR and FIR Filters - Advanced Solutions Nederland](https://www.advsolned.com/difference-between-iir-and-fir-filters-a-practical-design-guide/)
- [RTL-SDR Blog V4 Review - Elektor Magazine](https://www.elektormagazine.com/review/rtl-sdr-blog-v4-better-than-v3-review)
- [Pre-emphasis and De-emphasis - GeeksforGeeks](https://www.geeksforgeeks.org/pre-emphasis-and-de-emphasis/)
