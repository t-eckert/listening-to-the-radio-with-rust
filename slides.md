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

# Listening to the Radio with Rust

Thomas Eckert

---

<!-- speaker_note: Keep this casual. You're sharing a hobby, not lecturing. -->

About Me
========

Software engineer at Redpanda. Background in physics.

I'm always picking up new hobbies.

<!-- pause -->

A few months ago I got into software-defined radio.

When I find something cool, my first instinct is to share it.

<!-- pause -->

This is a technology that is at a _super_ accessible price point, but has a bit of a learning curve to get started.

---

What We'll Cover
================

<!-- incremental_lists: true -->

1. **Physics** — what is physically happening in an antenna
2. **Instrumentation** — how a $30 dongle digitizes radio waves
3. **Interpretation** — how different math extracts different signals
4. **Application** — live demos: FM, AM, and aircraft tracking

---

<!-- jump_to_middle -->

Let's start with the physics.

---

The Transmitter
===============

<!-- speaker_note: Use the bobber metaphor. Slow down here. Let the audience build the mental image. -->

Imagine a bobber sitting in a pool of still water.

<!-- pause -->

You push it up and down. It oscillates. Waves radiate outward across the surface.

<!-- pause -->

This is what happens when charged particles accelerate.

An electron moving up and down in a wire creates **electromagnetic waves** that radiate outward through space.

---

The Receiver
============

Now imagine a second bobber, sitting in the same pool some distance away.

<!-- pause -->

The waves reach it. It begins to oscillate too — driven by the energy carried in the waves.

<!-- pause -->

This is the receiving antenna.

Electrons in the metal are pushed up and down by the incoming EM wave.

The antenna converts the wave back into electrical current.

---

<!-- jump_to_middle -->

<!-- speaker_note: Run wave-demo here. Let the animation play while you narrate the summary. -->

Electrons moving up and down in a transmitting antenna create waves.

<!-- pause -->

Those waves push electrons up and down in a receiving antenna.

<!-- pause -->

Everything that follows is about measuring and interpreting that movement.

---

The RTL-SDR
===========

<!-- speaker_note: Show the dongle here. Hold it up. -->

**Traditional radio:** one hardware circuit, one purpose.

<!-- pause -->

**Software-defined radio:** digitize a chunk of spectrum, do everything else in software.

<!-- pause -->

Same dongle receives FM, AM, aviation, ADS-B —
just change the frequency and the code.

---

Two Chips
=========

```
    ┌─────────────────────────────────────────────────┐
    │                  RTL-SDR Dongle                 │
    │                                                 │
    │   ┌───────────┐          ┌──────────────┐       │
  ──┤   │   R828D   │─────────▶│  RTL2832U    │──── USB ──▶  your code
    │   │  (tuner)  │  analog  │   (ADC)      │       │
    │   └───────────┘          └──────────────┘       │
    │    shift freq              sample to            │
    │    to baseband             digital bytes        │
    └─────────────────────────────────────────────────┘
```

<!-- pause -->

**R828D tuner** — selects which part of the spectrum to listen to.

<!-- pause -->

**RTL2832U** — 8-bit ADC, sends digital data over USB.

---

The Tuner
=========

The antenna picks up _everything_ — FM, AM, aviation, cell towers, Wi-Fi — all at once.

<!-- pause -->

The R828D's job is to select a narrow slice of that spectrum.

It shifts your chosen frequency down to **baseband** — a low frequency
centered around zero that the ADC can sample.

<!-- pause -->

Think of it like a radio dial: you're not filtering out other stations,
you're sliding the window so your station lands at zero.

---

rtl_tcp
=======

The dongle streams raw bytes over USB.

`rtl_tcp` is a small server that forwards those bytes over TCP.

<!-- pause -->

```
RTL-SDR → USB → rtl_tcp → TCP → your code
```

<!-- pause -->

This means any program on the network can connect and receive samples.

All the demos tonight read from `rtl_tcp`.

---

<!-- jump_to_middle -->

What does this digital signal look like?

---

I and Q
=======

A radio signal at a single frequency is a cosine wave:

`signal(t) = A · cos(2π·f·t + φ)`

<!-- pause -->

The tuner shifts this to baseband. To capture both the amplitude _and_ the phase,
it samples the signal on **two axes**:

<!-- pause -->

- **I (in-phase)** — the cosine component: `A · cos(φ)`
- **Q (quadrature)** — the sine component: `A · sin(φ)`

<!-- pause -->

Together, `I + jQ` is a complex number that encodes
both the **amplitude** and the **phase** of the signal at each moment.

---

Points on the Complex Plane
============================

<!-- speaker_note: This is the key conceptual slide. Make sure the audience gets this before moving on. -->

The dongle outputs pairs of bytes: **I, Q, I, Q, ...**

Each pair is a point on the complex plane.

<!-- pause -->

These points trace rotation around the origin.

<!-- pause -->

**Speed of rotation** → proportional to the **frequency** of the wave.

<!-- pause -->

**Distance from origin** → proportional to the **amplitude** of the wave.

<!-- pause -->

This is the key insight.
Everything we do with SDR is about measuring these two properties.

---

<!-- jump_to_middle -->

<!-- speaker_note: Run iq-demo here. Use arrow keys to change frequency and amplitude. Show how faster rotation = higher frequency, larger circle = higher amplitude. -->

Let's see this.

---

Reading the Bytes
=================

The IQ data is already there — but the ADC encodes it as
unsigned 8-bit integers (0–255), not floats.

We just need to center and scale:

```rust
pub fn bytes_to_iq(raw: &[u8]) -> Vec<IqSample> {
    raw.chunks_exact(2)        // take pairs: [I, Q], [I, Q], ...
        .map(|pair| {
            IqSample::new(
                (pair[0] as f32 - 127.5) / 127.5,  // I: center and scale
                (pair[1] as f32 - 127.5) / 127.5,  // Q: center and scale
            )
        })
        .collect()
}
```

<!-- pause -->

`127` → `0.0`. `255` → `1.0`. `0` → `-1.0`.

<!-- pause -->

What matters is what they _mean_.

---

<!-- jump_to_middle -->

<!-- speaker_note: Switch to iq-print demo. Let the audience see raw IQ data streaming in from the dongle. -->

Let's see the real thing.

---

Many Signals, One Idea
=======================

All of the different signals sent over radio are interpreted
through different methods of **demodulation**.

<!-- pause -->

They all come back to measuring either
the **rotation speed** or the **distance from the origin** — or both.

---

Magnitude-Based Signals
========================

How far is the point from the origin? → `s.norm()`

| Signal | Frequency | What you get |
|--------|-----------|-------------|
| AM broadcast | 530–1700 kHz | Audio |
| ADS-B | 1090 MHz | Aircraft position, altitude, speed |
| OOK | 315/433 MHz | Garage doors, weather stations |
| NOAA APT | 137 MHz | Satellite weather images |

---

Phase-Based Signals
====================

How fast is the point rotating? → `(s * prev.conj()).arg()`

| Signal | Frequency | What you get |
|--------|-----------|-------------|
| FM broadcast | 88–108 MHz | Audio |
| FSK (pagers, telemetry) | Various | Digital bits |
| AIS (ships) | 162 MHz | Ship position and identity |
| POCSAG pagers | 148/152 MHz | Text messages |
| DMR/P25 | Various | Digital voice |

---

Why Rust?
=========

At 2.4 million samples per second, every sample gets **~400 ns** of processing time.

<!-- pause -->

- **No garbage collector** — no surprise pauses that drop samples
- **Zero-cost iterators** — the DSP pipeline compiles to tight loops
- **`num-complex`** — complex math feels native: `sample * prev.conj()`
- **Fearless concurrency** — read samples on one thread, play audio on another

<!-- pause -->

Performance comparable to C, with convenience and readability I prefer.

---

The Ecosystem
=============

| Crate | What it does |
|-------|-------------|
| `rtl-sdr-rs` | Pure Rust driver for the RTL-SDR dongle |
| `num-complex` | Complex number types — `I + jQ` just works |
| `rustfft` | Fast Fourier Transform for spectrum analysis |
| `cpal` | Cross-platform audio output |
| `ratatui` | Terminal UIs for visualizations |

<!-- pause -->

No C dependencies. Everything compiles with `cargo build`.

---

Every SDR Application
=====================

```
IQ samples → demodulate → process
```

<!-- pause -->

The demodulation step extracts a meaningful signal from the IQ data.

The processing step does something with it.

<!-- pause -->

- FM radio: demodulate phase → **play audio**
- AM radio: demodulate magnitude → **play audio**
- ADS-B: demodulate magnitude → **decode aircraft positions**

---

<!-- jump_to_middle -->

Let's hear some signals.

<!-- speaker_note: Start FM receiver demo. Audio should play through laptop speakers. Try 106.1 (CHEZ) or 100.3 (Majic). -->

---

FM Radio — The Pipeline
=======================

```
IQ samples (2 MHz)
  → low-pass filter + decimate 8x (256 kHz)
  → FM demodulate
  → low-pass filter + decimate to 48 kHz
  → de-emphasis
  → speakers
```

---

FM — Step 1: Filter
====================

The antenna hears every station at once.
We filter to just our channel (~200 kHz wide) and decimate 8x.

```rust
// The antenna picks up everything — all stations at once.
// Filter to just our channel (~200 kHz wide),
// then keep every 8th sample (2 MHz → 256 kHz).
// Fewer samples = less work for every step after this.
let filtered_iq = iq_filter.process(&iq_buf[..n]);
```

<!-- pause -->

2 million samples per second → 256 thousand. Much less work for the next steps.

---

FM — Step 2: Demodulate
========================

```rust
pub fn process(&mut self, input: &[Complex<f32>]) -> Vec<f32> {
    let mut output = Vec::with_capacity(input.len());

    for &sample in input {
        // Multiply by the conjugate of the previous sample.
        // This gives us the phase *difference* between them.
        let product = sample * self.prev.conj();

        // Extract the angle — that's the instantaneous frequency.
        let phase = product.im.atan2(product.re);

        // Scale it. This is now an audio sample.
        output.push(phase * self.gain);
        self.prev = sample;
    }

    output
}
```

<!-- pause -->

The audio _is_ the rate of phase change — the **rotation speed**.

---

FM — Step 3: Filter and Play
=============================

```rust
// We have 256,000 audio samples per second,
// but speakers only need 48,000.
// Filter out frequencies above 15 kHz (human hearing),
// then keep every 5th sample.
let mut audio = audio_filter.process_real(&audio_raw);

// FM stations boost high frequencies before transmitting.
// De-emphasis undoes that, restoring the original balance.
deemphasis.process(&mut audio);

// Send to speakers.
ring_buffer.push(&audio);
```

<!-- pause -->

That's the whole FM receiver.

---

AM Radio — The Pipeline
=======================

```
IQ samples (1 MHz)
  → low-pass filter + keep every 8th sample (128 kHz)
  → AM demodulate
  → low-pass filter + keep every 3rd sample (48 kHz)
  → speakers
```

<!-- pause -->

Same structure as FM. Different demodulation.

---

AM — Demodulation
=================

```rust
pub fn process(&self, input: &[Complex<f32>]) -> Vec<f32> {
    input.iter()
        .map(|s| s.norm())  // sqrt(I² + Q²) — distance from origin
        .collect()
}
```

<!-- pause -->

That's it. The audio _is_ the distance from the origin — the **amplitude**.

<!-- speaker_note: Switch to AM receiver demo. Tune to 118.8 MHz (YOW tower). Vertical antenna. You may hear ATC in English or French — Ottawa is bilingual. -->

---

AM — The Full Loop
==================

```rust
// Filter to just our channel, keep every 8th sample
let filtered_iq = iq_filter.process(&iq_buf[..n]);

// AM demodulate — magnitude of each IQ sample
let audio_raw = am_demod.process_ac_coupled(&filtered_iq);

// Keep every 3rd sample to get to 48 kHz for speakers
let mut audio = audio_filter.process_real(&audio_raw);

// Send to speakers
ring_buffer.push(&audio);
```

<!-- pause -->

Compare to FM: the only difference is one line — `s.norm()` instead of `(s * prev.conj()).arg()`.

---

<!-- jump_to_middle -->

Both FM and AM produce audio.

<!-- pause -->

FM uses the **speed of rotation** around the origin.

AM uses the **distance from the origin**.

<!-- pause -->

Same IQ data, different interpretation.

---

Antenna Length
==============

<!-- speaker_note: Start swapping the antenna here. Remove the long dipole, attach the short 7cm antenna. Talk through this while your hands are busy. -->

An antenna works best when its length is related to the wavelength of the signal.

<!-- pause -->

The sweet spot: **1/4 of the wavelength.**

At this length, the antenna resonates — electrons oscillate with maximum efficiency.

---

Why Not Longer?
===============

If the antenna is too long relative to the wavelength,
**eddy currents** form.

<!-- pause -->

Parts of the antenna work against each other — current flows
in opposing directions, canceling out the signal.

<!-- pause -->

This is why you don't use the same antenna for everything.

- FM (88 MHz) → wavelength ~3.4 m → dipole ~85 cm per side
- ADS-B (1090 MHz) → wavelength ~27 cm → antenna ~7 cm

<!-- pause -->

I just swapped antennas.
This short one is about 7 cm — a quarter wavelength at 1090 MHz.

---

<!-- jump_to_middle -->

<!-- speaker_note: Start ADS-B decoder demo. Show the Ottawa area map with aircraft appearing. -->

Let's see what's flying overhead.

---

ADS-B
=====

Every aircraft with a transponder broadcasts its position,
altitude, speed, and callsign. **Twice per second. Unencrypted.**

---

ADS-B — The Pipeline
=====================

```
IQ samples (2.4 MHz)
  → magnitude (same s.norm() as AM)
  → detect preamble pattern
  → extract 112 bits from pulse positions
  → CRC check
  → decode: callsign, lat, lon, altitude, speed
  → store in database
```

---

ADS-B — Demodulation
=====================

```rust
// Same operation as AM — just take the magnitude.
// ADS-B is on/off keying: high magnitude = 1, low = 0.
let mag: Vec<f32> = iq_buf[..n].iter()
    .map(|s| s.norm())
    .collect();
```

<!-- pause -->

Same `s.norm()` as AM. But instead of audio, the pattern of
high and low values encodes digital data.

---

ADS-B — Decode
==============

```rust
// Look for the ADS-B preamble pattern in the magnitude data
let raw_messages = demod.process(&mag);

for bits in &raw_messages {
    // Try to decode the 112-bit message
    if let Some(msg) = decode_message(bits) {
        // Extract position, altitude, callsign, speed...
        // and store in a SQLite database
        db.upsert(&aircraft)?;
    }
}
```

<!-- pause -->

Each message is a 120 μs burst — blink and you'd miss it.
But at 2.4 million samples per second, we catch every one.

---

<!-- jump_to_middle -->

Every plane in the sky above us is announcing itself right now.

<!-- pause -->

With a $30 dongle and a 7 cm antenna, we can see them all.

---

What We Covered
===============

<!-- incremental_lists: true -->

- **EM waves**: electrons oscillating in antennas, creating and receiving waves
- **The RTL-SDR**: two chips that digitize radio into IQ samples
- **IQ samples**: points on the complex plane — rotation speed is frequency, distance is amplitude
- **Demodulation**: FM (phase), AM (magnitude), ADS-B (magnitude)
- **Antennas**: why length matters and quarter-wavelength resonance

---

Getting Started
===============

**Hardware:**
- RTL-SDR Blog V4 (~$30)
- Dipole antenna kit (~$10)

**Software:**
- `rtl-sdr-rs`, `cpal`, `num-complex`, `ratatui`

**All code from this talk:**
- `github.com/t-eckert/listening-to-the-radio-with-rust`

<!-- pause -->

The airwaves are public. The signals are free.
The tools are open source.

**Go listen.**
