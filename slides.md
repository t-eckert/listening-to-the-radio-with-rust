---
title: Listening to the Radio with Rust
sub_title: RustConf 2026, Montreal, 9 September 2026
author: Thomas Eckert
theme:
  name: terminal-dark
options:
  end_slide_shorthand: true
---

<!-- jump_to_middle -->

<!-- speaker_note: PANIC CARD - If you lose your place, say one of these. (1) "So, what does this mean practically?" to transition to the next demo. (2) "Let me show you." to switch to any demo. (3) "The key idea is..." rotation speed is frequency, distance is amplitude. (4) "Let's go back to the IQ plane." to re-anchor on the core concept. You know this material. You built every demo. The audience is on your side. -->

# Listening to the Radio with Rust

Thomas Eckert

---

<!-- speaker_note: Keep this casual. You are sharing a hobby, not lecturing. Anchor quote - "I'm not an expert. This is something I've been playing with for a few months. When I find something cool, my first instinct is to share it." If you drift - You are here to share something cool. That is it. -->

About Me
========

Software engineer at Redpanda Data. Background in physics.

I'm always picking up new hobbies: cycling, climbing, homelab, watercolor, and now...

<!-- pause -->

software-defined radio.

<!-- pause -->

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

<!-- speaker_note: Anchor quote - "Imagine a bobber in a pool of still water." Use the bobber metaphor. Slow down here. Let the audience build the mental image. If you drift - Two bobbers. One makes waves, one receives them. Electrons up and down. -->

The Transmitter
===============

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

**Software-defined radio:** digitize a chunk of spectrum, do everything else in software.

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

<!-- speaker_note: Anchor quote - "The dongle gives you pairs of bytes. Each pair is a point on the complex plane." The phase changes over time. At each instant, we project the signal onto two axes, cosine and sine. That gives us two numbers, I and Q. That is why they come in pairs. If you drift - Points on a plane. Speed equals frequency. Distance equals amplitude. That is the whole game. -->

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

<!-- speaker_note: This is the key conceptual slide. Make sure the audience gets this before moving on. Frequency is rotation speed. Amplitude is distance from center. Everything we do with SDR is about measuring these two properties. -->

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

<!-- speaker_note: Run iq-demo here. Use arrow keys to change frequency and amplitude. Show how faster rotation equals higher frequency, larger circle equals higher amplitude. Let the audience see the connection. -->

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

---

<!-- jump_to_middle -->

<!-- speaker_note: Switch to iq-print demo. Keep it brief, 5-10 seconds. These are the actual numbers coming off the dongle right now. Every demo that follows processes these. Then move on. -->

Let's see the real thing.

---

<!-- speaker_note: Anchor quote - "All demodulation comes back to measuring rotation speed or distance from the origin." If you drift - Two columns. Magnitude or phase. Everything fits in one of them. -->

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

<!-- pause -->

Tools like GNURadio and SDR++ are more capable than what I've built.

But writing it yourself is how you _understand_ what's happening.

---

<!-- jump_to_middle -->

<!-- speaker_note: Anchor quote - "Multiply by the conjugate of the previous sample. Take the angle. That is FM demodulation." Start FM receiver demo. Try 97.7 (CHOM, rock) or 95.9 (Virgin). Verify at the venue. Let it play for a few seconds. Let the audience hear it. If you drift - One line of math. Phase change is audio. Play the music, show the code. -->

Let's hear some signals.

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

<!-- speaker_note: Anchor quote - "AM is even simpler. Just take the magnitude. sqrt of I squared plus Q squared. That is it." If you drift - Magnitude equals AM. One function call. Then contrast with FM. -->

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

<!-- pause -->

Let's tune to 119.9 MHz — Montréal-Trudeau tower.

In Canada, receiving is legal. The law restricts transmitting
and sharing private communications, but ATC is a public broadcast.

<!-- speaker_note: Switch to AM receiver demo. Tune to 119.9 MHz (CYUL main tower). Backups if it is quiet - 119.3 (north tower), 118.9 (south arrival). VERIFY ALL THREE AT THE VENUE the day before; these are from the published CYUL chart, not measured. Vertical antenna. You may hear ATC in English or French. If tower is silent for 10+ seconds, explain that ATC is bursty and move to the AM code slides while waiting. -->

---

AM — The Full Loop
==================

```rust
// Filter to just our channel, keep every 8th sample
let filtered_iq = iq_filter.process(&iq_buf[..n]);

// AM demodulate — magnitude of each IQ sample,
// with DC offset removed so audio centers on zero
let audio_raw = am_demod.process(&filtered_iq);

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

<!-- speaker_note: Anchor quote - "Everything so far has been listening. This one is about knowing what time it is." The shift - other demos turn radio into sound or data. This one turns radio into a clock. If you drift - time signal stations broadcast atomic time, continuously, one way, no network. -->

Time From the Sky
=================

Everything so far turned radio into **sound** or **data**.

<!-- pause -->

This one turns radio into a **clock**.

<!-- pause -->

Time signal stations broadcast the current time, continuously,
straight from a caesium atomic clock.

One-way. No network. No handshake. Nothing to log into.

---

<!-- speaker_note: This is the "why should I care" slide. Aim it at the embedded and infrastructure people in the room - they have all met a drifting RTC. Do not oversell it against NTP; the point is the cases where NTP is not reachable. If you drift - no network, still needs the time. -->

Why You'd Still Want This
=========================

Your hardware has no WiFi, no cell, no GPS.
How does it learn what time it is?

<!-- incremental_lists: true -->

- A sensor in a mine, a basement, a ship's hull
- An air-gapped machine that will never reach an NTP server
- A real-time clock that drifts seconds a week
- GPS solves this — but it needs a view of the sky, and it is trivially jammed

<!-- pause -->

Shortwave refracts off the ionosphere. It arrives from
**thousands of kilometres away, through walls.**

---

<!-- speaker_note: This is the emotional beat of the talk. Slow down and let the date land before the last line. Anchor quote - "I wrote a decoder for a radio station that doesn't exist anymore." If you drift - CHU ran 88 years, 15 km from my desk, and went silent two months before this talk. -->

CHU — Ottawa, 1938–2026
=======================

Fifteen kilometres from my desk in Ottawa: **CHU**,
run by the National Research Council.

3.330, 7.850, 14.670 MHz. Caesium clocks. Broadcasting since **1938**.

<!-- pause -->

I wrote a decoder for it — Bell 103 FSK, 300 baud, BCD time code.

<!-- pause -->

On **22 June 2026**, the NRC shut it off. After 88 years.

<!-- pause -->

I pointed the receiver at 7.850 MHz and found
**noise where a station used to be.**

---

<!-- speaker_note: Demo - spectrum view at 5.000 MHz, point at the single line. MEASURED on the office desk with a 2 m whip - carrier sits 10-14 dB above noise, and the sidebands do NOT make it. You will SEE the carrier, you will NOT hear the ticks. Do not promise audio. HF fades minute to minute; if the line is missing, say "that's shortwave" and move on - the story carries this section without the demo. -->

WWV — Fort Collins, Colorado
============================

CHU is gone. **WWV** is still transmitting. 2,900 km from my desk.

<!-- pause -->

The R820T tuner cannot go below 24 MHz. So we bypass it completely —
**direct sampling**, where the antenna feeds the ADC directly.

```bash
rtl_sdr -f 5000000 -s 250000 -D
```

<!-- pause -->

The carrier is at 5.000000 MHz. Not approximately — *exactly*,
because it comes off the same kind of atomic standard it is reporting.

---

<!-- jump_to_middle -->

<!-- speaker_note: This is the bridge into antenna length, and it is doing real work - the honest limitation IS the setup for the next section. Say the 15 metres and the 2 metres slowly, hold up the whip on "my antenna is 2 metres." Then walk straight into antenna theory. -->

I can see the carrier. I can't hear the ticks.

<!-- pause -->

A quarter wavelength at 5 MHz is **15 metres**.

My antenna is 2 metres.

<!-- pause -->

Which brings us to antenna length.

---

<!-- speaker_note: Anchor quote - "An antenna resonates at a quarter of the wavelength." Swap the antenna during this section. The physical act reinforces the point. If you drift - Quarter wavelength. Too long equals destructive interference. Show the short antenna. -->

Antenna Length
==============

An antenna works best when its length is related to the wavelength of the signal.

<!-- pause -->

The sweet spot: **1/4 of the wavelength.**

At this length, the antenna resonates — electrons oscillate with maximum efficiency.

---

Why Not Longer?
===============

If the antenna is too long relative to the wavelength,
you get **destructive interference**.

<!-- pause -->

Current flows in opposite directions in different parts of the antenna —
their radiated fields cancel each other out.

<!-- pause -->

This is why you don't use the same antenna for everything.

- FM (88 MHz) → wavelength ~3.4 m → dipole ~85 cm per side
- ADS-B (1090 MHz) → wavelength ~27 cm → antenna ~7 cm

<!-- pause -->

I just swapped antennas.
This short one is about 7 cm — a quarter wavelength at 1090 MHz.

---

<!-- jump_to_middle -->

<!-- speaker_note: Anchor quote - "Every plane in the sky is announcing itself right now." Start ADS-B decoder demo. Run adsb-decoder, then in a second terminal run task tracker (it defaults to the montreal region now; the bare binary still defaults to ottawa, and the wrong region renders an empty map with no error). Aircraft appear on the Montreal map with callsign, altitude, speed. Let it accumulate. Each new dot is satisfying. If you drift - Same math as AM, different protocol on top. Show the map. Let it fill in. -->

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

<!-- speaker_note: Anchor quote - "Everything I showed you today costs about $30 and runs on any laptop." Invitation - "The airwaves are public. The signals are free. The tools are open source. Go listen." Open discussion - pass around the hardware, ask if anyone has done amateur radio or SDR. If you drift - $30 dongle, a few crates, curiosity. Go listen. -->

Getting Started
===============

**Hardware:**
- RTL-SDR Blog V4 (~$30)
- Dipole antenna kit (~$10)

**No hardware yet?** Try `wave-demo` and `iq-demo` — they visualize
the core concepts with no dongle needed.

**All code from this talk:**
- `github.com/t-eckert/listening-to-the-radio-with-rust`

<!-- pause -->

The airwaves are public. The signals are free.
The tools are open source.

**Go listen.**
