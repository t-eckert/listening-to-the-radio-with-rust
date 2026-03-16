# How the Demos Work

A walk through the signal processing chain in each demo, from antenna to output.
Written as a learning document — explaining not just what each step does, but why.

---

## The Shared Foundation: IQ Samples

Every demo starts the same way. The RTL-SDR dongle is tuned to a frequency, and it
outputs a stream of **IQ samples**.

### What's actually happening in the hardware

The antenna picks up electromagnetic waves across a wide range of frequencies. The
R828D tuner chip inside the dongle does two things:

1. **Amplifies** the signal (the gain we set)
2. **Downconverts** — it mixes the signal with a local oscillator to shift everything
   down so that your target frequency lands at 0 Hz

After downconversion, the RTL2832U chip digitizes the result with an 8-bit ADC. The
output is pairs of bytes: one I (in-phase) and one Q (quadrature).

### Why pairs? Why not just one number?

A single number per sample (like a microphone recording) can only tell you the
amplitude at each instant. It can't tell you the *direction* of a frequency offset.
Imagine you're tuned to 100 MHz. A station at 100.1 MHz and a station at 99.9 MHz
are both 0.1 MHz away — with a single number, they'd look identical. You couldn't
tell them apart.

IQ samples solve this by capturing **two** components of the signal simultaneously:

- **I** = the component aligned with cosine (the "real" part)
- **Q** = the component aligned with sine (the "imaginary" part)

Together, they form a **complex number**: `s = I + jQ`. This is not an arbitrary
mathematical abstraction — it directly corresponds to how the hardware works. The
RTL2832U literally multiplies the digitized signal by a cosine (producing I) and a
sine (producing Q) at the same time. Two separate multiplication paths, 90 degrees
apart. That's what "quadrature" means.

Each IQ sample is a point on the complex plane. You can think of it as an arrow from
the origin with:

- A **length** (amplitude): `sqrt(I² + Q²)`
- An **angle** (phase): `atan2(Q, I)`

A signal above the center frequency makes this arrow spin counterclockwise. A signal
below it spins clockwise. The speed of rotation is the frequency offset. This is the
key insight that makes everything else work.

### The raw data format

The RTL-SDR outputs unsigned bytes:

```
[I₀] [Q₀] [I₁] [Q₁] [I₂] [Q₂] ...
```

Each byte is 0–255. We convert to floating point centered on zero:

```rust
let i = (raw_byte as f32 - 127.5) / 127.5;  // maps to roughly [-1, +1]
```

At 2.048 MHz sample rate, that's 2,048,000 complex samples per second, or about
4 MB/s of raw bytes flowing through USB into our code.

---

## Demo 1: FM Receiver

### The signal processing chain

```
Antenna → RTL-SDR (2.048 MS/s) → IQ samples
  → Low-pass filter + decimate (isolate the FM channel)
  → FM demodulate (extract audio from phase changes)
  → Low-pass filter + decimate (reduce to 48 kHz audio rate)
  → De-emphasis filter (compensate for FM pre-emphasis)
  → Speaker
```

Each step exists for a specific reason.

### Step 1: Low-pass filter the IQ signal

The RTL-SDR at 2.048 MS/s captures about 2 MHz of spectrum. If you're tuned to
100.3 MHz, you're seeing everything from roughly 99.3 to 101.3 MHz. There might be
multiple FM stations in that window (they're spaced 200 kHz apart).

The low-pass filter isolates just the one station you care about. We set the cutoff
at 100 kHz (half the 200 kHz FM channel width), which keeps only the signal within
±100 kHz of the center frequency.

**How the filter works**: It's a FIR (Finite Impulse Response) filter. The idea is
simple — each output sample is a weighted average of the surrounding input samples.
The weights (called "taps") are designed so that low frequencies pass through and
high frequencies are attenuated.

The taps are computed by the **windowed sinc method**:

1. Start with the ideal low-pass filter: a sinc function `sin(2πfx) / (πx)`. This
   is the mathematically perfect low-pass filter, but it's infinitely long.
2. Multiply by a Hamming window to make it finite. The window smoothly tapers to
   zero at the edges, which prevents sharp cutoffs from causing ringing artifacts.
3. Normalize so the taps sum to 1 (unity gain at DC).

With 51 taps, the filter examines 51 surrounding samples to compute each output.
More taps = sharper cutoff but more computation.

**Decimation**: After filtering, we don't need all 2.048 million samples per second
anymore — the signal is now only ~200 kHz wide. So we keep every Nth sample and
throw the rest away. This is safe because the filter already removed all the
energy above the new Nyquist limit. In our code, the `LowPassFilter` does filtering
and decimation in one step — it only computes the output for samples it's going to
keep, which saves a lot of work.

### Step 2: FM demodulation

This is the heart of the FM receiver, and it's surprisingly simple.

**What FM encoding is**: An FM transmitter takes the audio signal and uses it to
vary the *frequency* of the carrier wave. When the audio goes positive, the carrier
frequency increases. When negative, it decreases. The amplitude stays constant.

A broadcast FM station has a maximum deviation of ±75 kHz. So if you're tuned to
100.3 MHz, the carrier is wobbling between 100.225 and 100.375 MHz, and the pattern
of that wobbling IS the audio.

**How to extract it**: We need to measure the instantaneous frequency at each moment.
Since each IQ sample has a phase angle, and frequency is the rate of change of phase,
we can get the frequency by computing the **phase difference** between consecutive
samples.

The naive approach would be:

```
phase[n] = atan2(Q[n], I[n])
audio[n] = phase[n] - phase[n-1]
```

But this has a problem: phase wraps around at ±π. If the phase goes from +3.1 to
-3.1, the difference looks like -6.2 when it should be +0.08. You'd hear a loud
click every time the phase wraps.

**The conjugate multiply trick** avoids this entirely:

```rust
let product = sample * prev.conj();
let audio = product.arg();  // atan2(im, re)
```

Here's why this works. When you multiply two complex numbers, their angles add.
When you conjugate a complex number, its angle is negated. So:

```
angle(sample × conj(prev)) = angle(sample) + angle(conj(prev))
                            = angle(sample) - angle(prev)
                            = Δphase
```

The result `product` is a complex number whose angle is exactly the phase difference,
with no wrapping issues. Then `arg()` (which is `atan2(im, re)`) extracts that angle.

The output is proportional to the instantaneous frequency offset, which is
proportional to the audio. We multiply by a gain factor to normalize it:

```rust
let gain = sample_rate / (2π × max_deviation);
```

This scales the output so that ±75 kHz deviation maps to ±1.0.

**That's the entire FM demodulation algorithm.** One complex multiply, one atan2.

### Step 3: Low-pass filter + decimate to audio rate

After FM demodulation, we have audio samples at whatever rate we had after the first
decimation. This is usually much higher than we need for audio output (48 kHz). So
we filter again to keep only frequencies up to ~15 kHz (the audio bandwidth of FM
mono) and decimate down to 48 kHz.

### Step 4: De-emphasis filter

FM broadcasting uses a trick to reduce noise. High frequencies are boosted before
transmission (pre-emphasis) and attenuated in the receiver (de-emphasis). This
works because FM noise increases with frequency, so boosting highs before
transmission and cutting them after effectively reduces the noise at high audio
frequencies.

The de-emphasis filter is a simple single-pole IIR (Infinite Impulse Response) filter
with a time constant of 75 μs (in North America). It's one line:

```rust
self.prev += alpha * (sample - self.prev);
```

This is an exponential moving average. `alpha` controls the cutoff frequency —
smaller alpha means more smoothing (lower cutoff). At 75 μs, the cutoff is about
2,122 Hz. Everything above that rolls off at -6 dB per octave.

### Step 5: Audio output

The `cpal` crate provides a callback-based audio output. We set up a stream at
48 kHz, and it calls our callback whenever it needs more samples to play. The
callback pulls from a channel that the processing loop pushes to. If no data is
ready, it outputs silence (zeros).

### Threading model

The processing runs on the main thread in a simple loop: read IQ → filter → demod →
filter → de-emphasis → send to channel. The audio callback runs on a separate
thread managed by `cpal`. The `mpsc::sync_channel` connects them with a small buffer.

---

## Demo 3: Frequency Scanner

(Covering this before CHU since the scanner is simpler and running.)

### What it does

The scanner steps through a range of frequencies, measures the signal power at each
step, and identifies which frequencies have active transmissions.

### The sweep loop

```
For each frequency step:
  1. Retune the SDR to that frequency
  2. Read a chunk of IQ samples
  3. Compute the signal power using FFT
  4. Record the power level
After a complete sweep:
  5. Compute noise floor (median of all power readings)
  6. Detect signals (power readings that exceed noise floor + threshold)
  7. Consolidate adjacent detections into single peaks
```

### Why FFT instead of just averaging the power?

You could measure signal power simply: `mean(I² + Q²)` over all samples. This works,
but it has a problem. The RTL-SDR captures about 2 MHz of bandwidth at once. If your
step size is 100 kHz but your captured bandwidth is 2 MHz, most of what you're
measuring is noise from frequencies you don't care about. The signal you're looking
for might be a narrow 200 kHz channel within that 2 MHz window.

The **FFT** (Fast Fourier Transform) breaks the 2 MHz chunk into individual frequency
bins. With a 4096-point FFT at 2.048 MS/s, each bin is about 500 Hz wide. We can
then measure power in just the bins near the center frequency (where the signal of
interest is) and ignore the rest.

### How the FFT works (conceptually)

The FFT takes N time-domain IQ samples and transforms them into N frequency-domain
bins. Each bin tells you how much energy is present at a particular frequency offset
from the center.

```
Bin 0:     DC component (0 Hz offset — the center frequency itself)
Bin 1:     +fs/N Hz (one step above center)
Bin 2:     +2·fs/N Hz
...
Bin N/2:   +fs/2 Hz (Nyquist frequency — highest positive frequency)
Bin N/2+1: -fs/2 + fs/N Hz (wraps to most negative frequency)
...
Bin N-1:   -fs/N Hz (one step below center)
```

So the bins near index 0 and index N contain the frequencies closest to where we
tuned the SDR. This is why the scanner measures bins `1..N/4` and `3N/4..N` — these
are the frequencies within ±25% of the center, avoiding bin 0 (which has a DC spike
artifact from the RTL-SDR's local oscillator leakage).

### Windowing

Before computing the FFT, we multiply the samples by a **window function** (Hamming
window in our case). Why?

The FFT assumes the input signal repeats infinitely. But we only have a finite chunk.
The sharp edges where the chunk starts and ends act like sudden discontinuities,
which spread energy across all frequency bins — this is called **spectral leakage**.
A strong signal at one frequency smears into neighboring bins.

The window function smoothly tapers the edges to zero, eliminating the
discontinuity. The tradeoff is slightly reduced frequency resolution (the bins
become a little wider), but the leakage is dramatically reduced. For a scanner where
we care about measuring power accurately, this is essential.

### Power calculation

For each bin, the power is the magnitude squared of the complex FFT output:

```
power = re² + im²
```

We average this across our selected bins and normalize by dividing by N² (because
the FFT scales magnitudes by N, so power scales by N²). The result is converted to
decibels:

```
dB = 10 × log₁₀(power)
```

### Signal detection

After a complete sweep, the scanner has a power reading (in dB) for each frequency
step. To find signals:

1. **Compute the noise floor**: Take the median of all power readings. The median
   is robust against outliers — a few strong signals won't skew it the way a mean
   would. This gives us a baseline: the power level of "nothing interesting."

2. **Threshold**: Any reading that exceeds `noise_floor + threshold_dB` is considered
   a signal. With a 6 dB threshold, a signal needs to be about 4× the noise power
   to be detected.

3. **Peak consolidation**: An FM station occupies ~200 kHz, so it might appear in
   multiple adjacent 100 kHz steps. The scanner groups consecutive above-threshold
   steps and reports only the frequency with the highest power in each group.

### Why manual gain matters

With automatic gain control (AGC), the RTL-SDR adjusts its amplification at each
frequency to keep the signal level roughly constant. This is great for listening to
a single station (the FM receiver uses it), but terrible for scanning — it erases
the very differences you're trying to measure. An empty frequency gets boosted,
a strong station gets attenuated, and everything ends up at a similar power level.

With manual gain, the amplification is fixed. Empty frequencies produce low power
readings, and active stations produce high ones. The difference is what the scanner
detects.

---

## Demo 2: CHU Time Signal Decoder

(Not yet tested with live signals, but the code is written.)

### What CHU is

CHU is a shortwave time signal station operated by the National Research Council
of Canada. It's in Barrhaven, about 10 miles from downtown Ottawa. It broadcasts
the exact time on three frequencies: 3.330, 7.850, and 14.670 MHz. It's been
operating since 1938.

The signal carries the time in three ways simultaneously:

1. **Audible ticks**: A 1000 Hz tone pulsed once per second
2. **Voice announcements**: Bilingual (English/French) time announcements
3. **Digital time code**: FSK data bursts encoding the time in BCD

The decoder targets the digital time code — it's the most precise and the most
interesting from a signal processing perspective.

### The signal processing chain

```
Antenna → RTL-SDR (HF via V4 upconverter) → IQ samples
  → AM demodulate (extract the audio envelope)
  → Low-pass filter + decimate to 8 kHz
  → Goertzel tone detection (2025 Hz vs 2225 Hz)
  → UART framing (start bit, 8 data bits, 2 stop bits)
  → Nibble swap + BCD parsing
  → UTC time
```

### Step 1: AM demodulation

Unlike FM broadcast, CHU uses AM-compatible modulation (technically H3E — upper
sideband with reinserted carrier). For AM, the audio is encoded in the **amplitude**
of the carrier, not the frequency.

AM demodulation is even simpler than FM — just take the magnitude of each IQ sample:

```rust
audio = sqrt(I² + Q²)
```

This is called **envelope detection**. The magnitude of the complex signal IS the
audio signal. (We also subtract the mean to remove the DC offset from the carrier
itself, leaving just the audio modulation — that's the `process_ac_coupled` method.)

### Step 2: FSK demodulation with Goertzel

The digital time code is encoded as **FSK** (Frequency Shift Keying). Two audio
tones represent binary 0 and 1:

- **2225 Hz** = mark (binary 1)
- **2025 Hz** = space (binary 0)

This is the Bell 103 modem standard from the 1960s. At 300 baud, each bit lasts
1/300th of a second (about 3.33 ms, or 26.67 samples at 8 kHz).

To determine which tone is present in each bit period, we use the **Goertzel
algorithm**. It's like a tiny, targeted FFT that computes the energy at just one
specific frequency. We run two Goertzel filters — one at 2025 Hz and one at 2225 Hz
— and compare their outputs:

```rust
let mark_power = mark_detector.power(bit_samples);    // energy at 2225 Hz
let space_power = space_detector.power(bit_samples);  // energy at 2025 Hz
let bit = if mark_power > space_power { 1 } else { 0 };
```

**How Goertzel works**: It's a second-order IIR filter that resonates at the target
frequency. As you feed it samples, it accumulates energy at that frequency. After
processing all samples in the window, the final state gives you the magnitude.

```
s₀ = sample + coeff × s₁ - s₂
s₂ = s₁
s₁ = s₀
```

where `coeff = 2 × cos(2π × target_freq / sample_rate)`. The coefficient is
precomputed once. Then after all samples:

```
power = s₁² + s₂² - coeff × s₁ × s₂
```

This is equivalent to computing a single bin of the DFT, but much more efficient
when you only care about one or two frequencies.

### Step 3: UART framing

The raw bit stream needs to be organized into bytes. CHU uses **8N2** serial
framing — the same protocol used by old modems and serial ports:

```
[start=0] [d0] [d1] [d2] [d3] [d4] [d5] [d6] [d7] [stop=1] [stop=1]
```

The decoder is a state machine:

1. **WaitingForStart**: Watch for a 0 bit (start bit). In idle, the line sits at
   mark (1), so a transition to 0 signals the beginning of a byte.
2. **ReceivingBits**: Collect 8 data bits, LSB first.
3. **WaitingForStop**: Verify 2 stop bits (both must be 1). If either is 0, it's
   a framing error — discard and go back to waiting.

### Step 4: Burst assembly and BCD decoding

CHU transmits data bursts at seconds 31–39 of each minute. Each burst contains 10
bytes: 5 data bytes followed by 5 redundancy bytes.

**Nibble swap**: Each received byte has its high and low nibbles swapped relative
to the logical BCD digits. `0x06` in the wire becomes `0x60` after swap, which
represents "6" and "0". This is just a quirk of the encoding.

```rust
fn nibble_swap(byte: u8) -> u8 {
    (byte >> 4) | (byte << 4)
}
```

**Format A** (seconds 32–39): After nibble swap, the 5 data bytes encode:

```
Byte 0: [6][d₁]     framing code (always 6) + hundreds digit of day-of-year
Byte 1: [d₂][d₃]    tens and units of day-of-year
Byte 2: [h₁][h₂]    tens and units of hour (UTC)
Byte 3: [m₁][m₂]    tens and units of minute
Byte 4: [s₁][s₂]    tens and units of second
```

The framing code `6` in the first nibble serves as validation — if it's not 6,
the burst is corrupt. The 5 redundancy bytes are identical to the data bytes,
providing a second level of validation.

**Format B** (second 31): Contains the year, DUT1 correction, and other metadata.
The redundancy bytes are the one's complement (bitwise NOT) of the data bytes.

CHU transmits 8 Format A bursts per minute (seconds 32–39), each with the second
field incrementing. One valid burst gives you the complete UTC time. The redundancy
means you can tolerate significant interference and still decode correctly.

### HF reception on the V4

CHU's frequencies (3.33, 7.85, 14.67 MHz) are all below the R828D tuner's minimum
frequency of 24 MHz. The RTL-SDR Blog V4 solves this with a built-in **upconverter**
— a SA612 mixer that shifts HF signals up by 28.8 MHz. So CHU at 7.850 MHz becomes
36.650 MHz, which the tuner handles normally.

The `rtl_sdr_blog` feature flag in `rtl-sdr-rs` enables automatic frequency
translation — you tune to 7,850,000 Hz and the driver adds the 28.8 MHz offset
transparently.

---

## The Shared SDR Library

The `sdr` crate provides the building blocks that all three demos share.

### Source abstraction

The `SdrSource` trait abstracts over three ways to get IQ data:

- **UsbSource**: Direct USB connection via `rtl-sdr-rs`. Talks to the dongle's
  registers over USB bulk transfers.
- **TcpSource**: Connects to an `rtl_tcp` server over TCP. The protocol is trivial:
  12-byte header on connect, 5-byte commands to control the dongle, then a firehose
  of raw IQ bytes.
- **FileSource**: Reads pre-recorded IQ data from a file. Paces the output to
  simulate real-time streaming. Critical for demo fallback — if the dongle fails
  on stage, swap one argument and the code runs identically on recorded data.

All three produce the same thing: a stream of `Complex<f32>` samples. The processing
code doesn't know or care where they came from.

### DSP modules

- **convert**: Raw `u8` bytes to `Complex<f32>` IQ samples
- **filter**: FIR low-pass filter with decimation (windowed sinc design)
- **fm**: FM demodulator (conjugate multiply method)
- **am**: AM demodulator (envelope detection)
- **deemphasis**: Single-pole IIR de-emphasis filter
- **tone**: Goertzel algorithm for single-frequency detection, plus frequency shifting
- **power**: Signal power computation and dB conversion
- **window**: Hamming and Blackman window functions for FFT
