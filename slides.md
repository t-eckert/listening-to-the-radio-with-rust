---
title: Listening to the Radio with Rust
sub_title: Ottawa Systems, 24 March 2026
author: Thomas Eckert
theme:
  name: terminal-dark
options:
  end_slide_shorthand: true
---

<!-- jump_to_middle -->

<!-- speaker_note: Open cold. No intro. Start FM receiver BEFORE this slide. Audio should already be playing from the laptop speakers when the audience sees this. Let it play for a few seconds. -->

_This is live._

<!-- pause -->

An antenna, a **$30 USB dongle**, and about 50 lines of Rust.

---

The Code
========

```rust
pub fn process(&mut self, input: &[Complex<f32>]) -> Vec<f32> {
    let mut output = Vec::with_capacity(input.len());

    for &sample in input {
        let product = sample * self.prev.conj();
        let phase = product.im.atan2(product.re);
        output.push(phase * self.gain);
        self.prev = sample;
    }

    output
}
```

<!-- pause -->

Multiply by the conjugate of the previous sample. Take the angle.

The audio _is_ the rate of phase change.

---

<!-- jump_to_middle -->

FM is the obvious one.

<!-- pause -->

But the spectrum doesn't stop at 108 MHz.

---

What's in the Air Right Now
============================

<!-- incremental_lists: true -->

- **88 – 108 MHz** — FM broadcast. The thing we just heard.
- **118 – 137 MHz** — Aviation AM. Tower, ground, approach.
- **144 – 148 MHz** — 2-meter ham band. Local repeaters.
- **1090 MHz** — ADS-B. Every aircraft broadcasting its position. Unencrypted.
- **3.33 / 7.85 / 14.67 MHz** — Shortwave. We'll come back to these.

---

<!-- jump_to_middle -->

All of this is in the air _in this room_ right now.

<!-- pause -->

The only difference between hearing it and not
is whether you have the right tool.

---

What is SDR?
============

**Traditional radio:** one hardware circuit, one purpose.

<!-- pause -->

**Software-defined radio:** digitize a wide chunk of spectrum,
do everything else in software.

<!-- pause -->

Same dongle receives FM, aviation, ADS-B, shortwave —
just change the frequency and the processing code.

<!-- pause -->

Originally a $20 DVB-T TV tuner.
Hackers figured out you could grab raw samples from the ADC.

---

Antenna to Iterator
===================

```
Antenna → R828D tuner → RTL2832U → USB → your code
           (amplify,     (8-bit ADC,
           downconvert)   2.4 MS/s)
```

<!-- pause -->

- **R828D:** LNA + mixer + PLL. Shifts RF down to baseband.
- **RTL2832U:** 8-bit ADC. Outputs interleaved bytes: I, Q, I, Q, ...
- **USB bulk transfers:** 15 in-flight buffers of 256 KB. Keep up or drop samples.

<!-- pause -->

**8 bits is the constraint.** ~40 dB dynamic range.

---

The Data
========

Raw: unsigned `u8` pairs [0, 255]. Each pair is one complex sample.

```rust
pub fn bytes_to_iq(raw: &[u8]) -> Vec<IqSample> {
    raw.chunks_exact(2)
        .map(|pair| {
            IqSample::new(
                (pair[0] as f32 - 127.5) / 127.5,
                (pair[1] as f32 - 127.5) / 127.5,
            )
        })
        .collect()
}
```

<!-- pause -->

At 2.4 MS/s: **4.8 MB/s** of IQ data flowing through your Rust code.

---

Two Ways to Connect
===================

```
USB direct (rtl-sdr-rs)       Network (rtl_tcp)
  RTL-SDR → USB → read()       RTL-SDR → Pi → TCP → read()
```

<!-- pause -->

Both produce the same `&[u8]` stream.

rtl_tcp: 12-byte header, 5-byte commands, then a firehose of IQ bytes.

---

<!-- jump_to_middle -->

So we heard FM.

<!-- pause -->

Let's see what else is out there.

<!-- speaker_note: Switch to freq-scanner demo. Sweep FM band first, then aviation band. -->

---

Frequency Scanner
=================

<!-- speaker_note: Run the scanner TUI live. Show FM band first (88-108 MHz), then switch to aviation. -->

The sweep loop:

```rust
// For each frequency step:
source.set_frequency(freq)?;   // retune the hardware
let n = source.read(&mut buf)?; // grab samples
scanner.measure(&buf[..n]);     // FFT → power
scanner.step();                  // advance
```

<!-- pause -->

1024-point FFT, Hamming window, magnitude squared.

The PLL settling time (~5 ms) is the bottleneck, not the Rust code.

---

<!-- jump_to_middle -->

Remember those shortwave frequencies I skipped?

<!-- pause -->

**3.33 / 7.85 / 14.67 MHz**

<!-- pause -->

Let's go back to them.

---

CHU — Ottawa's Atomic Clock
============================

- Shortwave time signal station, operated by NRC Canada.
- Broadcasting continuously since **1938**.

<!-- pause -->

- Three frequencies. Three antennas. Three cesium atomic clocks.
- Transmitter site: Barrhaven. About 10 miles from here.

<!-- pause -->

- Every second: a 1000 Hz tick.
- Every minute: _"At the tone, Eastern Daylight Time will be..."_
- Seconds 31–39: a 300-baud FSK data burst encoding the exact time.

<!-- pause -->

- Carrier accuracy: **5 parts per trillion.** Derived from cesium.

---

The Decoding Chain
==================

```
7.850 MHz RF
  → AM demod (envelope detection)
  → audio stream
  → bandpass filter (2025–2225 Hz)
  → FSK discriminator
  → UART framing (8N2 @ 300 baud)
  → nibble swap
  → BCD parse
  → UTC time
```

---

AM Demodulation
===============

FM: multiply by conjugate, take the angle.

AM: just take the magnitude.

```rust
pub fn process(&self, input: &[Complex<f32>]) -> Vec<f32> {
    input.iter().map(|s| s.norm()).collect()
}
```

<!-- pause -->

`sqrt(I² + Q²)`. That's it.

---

FSK — A 1960s Modem
====================

CHU sends data as two tones: **2025 Hz** (space/0) and **2225 Hz** (mark/1).

That's Bell 103 modem encoding.

<!-- pause -->

Detect each tone with the Goertzel algorithm:

```rust
pub fn power(&self, samples: &[f32]) -> f32 {
    let mut s0: f32;
    let mut s1 = 0.0f32;
    let mut s2 = 0.0f32;

    for &sample in samples {
        s0 = sample + self.coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }

    s1 * s1 + s2 * s2 - self.coeff * s1 * s2
}
```

<!-- pause -->

Compare the two powers. Whichever is louder wins. One bit per 1/300th of a second.

---

UART Framing
============

```rust
enum DecoderState {
    WaitingForStart,
    ReceivingBits,
    WaitingForStop,
}
```

<!-- pause -->

Detect start bit → sample 8 data bits → check 2 stop bits.

You're building a software modem.

<!-- pause -->

10 bytes per burst, transmitted twice for redundancy.

Nibble-swap, parse BCD → **year, day-of-year, hour, minute, second.**

---

<!-- jump_to_middle -->

<!-- speaker_note: Switch to CHU decoder demo. Let it run. Let the ticks accumulate. Let the time appear. Take your time here. -->

Let's listen.

---

<!-- jump_to_middle -->

That time came from a cesium clock in Barrhaven,
through the air, into a $30 dongle,
through a few hundred lines of Rust.

<!-- pause -->

No internet. No GPS. No NTP.

<!-- pause -->

Just radio waves and math.

---

What We Built
=============

<!-- incremental_lists: true -->

- FM radio from one line of phase math.
- A spectrum scanner that makes the invisible visible.
- An atomic time decoder for a signal broadcasting since 1938.
- All of it: one USB dongle, pure Rust, no C dependencies.

---

Magnitude-Based Signals
========================

How long is the vector? → `s.norm()`

| Signal | Frequency | What you get |
|--------|-----------|-------------|
| AM broadcast | 530–1700 kHz | Audio |
| ADS-B | 1090 MHz | Aircraft position, altitude, speed |
| OOK | 315/433 MHz | Garage doors, weather stations |
| NOAA APT | 137 MHz | Satellite weather images |

<!-- pause -->

ADS-B: a 120 μs burst → magnitude → pulse positions → lat, lon, altitude.

Same `s.norm()` as AM. Different protocol on top.

---

Phase-Based Signals
====================

How fast is the vector spinning? → `(s * prev.conj()).arg()`

| Signal | Frequency | What you get |
|--------|-----------|-------------|
| FM broadcast | 88–108 MHz | Audio |
| FSK (CHU, pagers) | Various | Digital bits |
| AIS (ships) | 162 MHz | Ship position and identity |
| POCSAG pagers | 148/152 MHz | Text messages |
| DMR/P25 | Various | Digital voice |

<!-- pause -->

FSK is just FM where the "audio" is a square wave.

FM demod → threshold → bits → protocol.

---

Getting Started
===============

**Hardware:**
- RTL-SDR Blog V4 (~$30)
- Dipole antenna kit (~$10)

**Software:**
- `rtl-sdr-rs`, `rustfft`, `cpal`, `num-complex`
- `github.com/t-eckert/listening-to-the-radio-with-rust`

<!-- pause -->

The airwaves are public. The signals are free.
The tools are open source.

**Go listen.**
