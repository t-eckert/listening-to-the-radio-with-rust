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

<!-- jump_to_middle -->

Let's hear some signals.

<!-- speaker_note: Start FM receiver demo. Audio should play through laptop speakers. Try 106.1 (CHEZ) or 100.3 (Majic). -->

---

FM Demodulation
===============

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

The audio _is_ the rate of phase change — the **rotation speed**.

---

AM Demodulation
===============

```rust
pub fn process(&self, input: &[Complex<f32>]) -> Vec<f32> {
    input.iter().map(|s| s.norm()).collect()
}
```

<!-- pause -->

`sqrt(I² + Q²)`. Just take the magnitude.

The audio _is_ the distance from the origin — the **amplitude**.

<!-- speaker_note: Switch to AM receiver demo. Tune to 118.8 MHz (YOW tower). Vertical antenna. You may hear ATC in English or French — Ottawa is bilingual. -->

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

<!-- pause -->

ADS-B is a magnitude-based signal — on/off keying at 1090 MHz.

A 120 μs burst → magnitude → pulse positions → lat, lon, altitude.

<!-- pause -->

Same `s.norm()` as AM. Different protocol on top.

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
