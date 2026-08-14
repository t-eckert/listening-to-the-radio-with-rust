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

<!-- speaker_note: COLD OPEN. The FM receiver is ALREADY RUNNING before the emcee introduces you, audio muted at the mixer or volume down. You walk on, bring the volume up, and say NOTHING for ten seconds. Let the room hear music. Then - "That's CHOM, 97.7. It came out of the air, into a thirty dollar USB stick, through about forty lines of Rust that I wrote, and out of those speakers. I'm Thomas. Let me show you how." Then kill the audio and advance. -->

<!-- speaker_note: PANIC CARD - If you lose your place, say one of these. (1) "So, what does this mean practically?" to transition to the next demo. (2) "Let me show you." to switch to any demo. (3) "The key idea is..." rotation speed is frequency, distance is amplitude. (4) "Let's go back to the IQ plane." to re-anchor on the core concept. You know this material. You built every demo. The audience is on your side. -->

<!-- speaker_note: IF THE COLD OPEN FAILS - do not debug on stage. Say "Live RF. That's part of the fun, and we'll get it back later." Advance to the title and carry on. The pre-recorded IQ file is one keystroke away for the FM section proper. -->

97.7 MHz

---

<!-- jump_to_middle -->

<!-- speaker_note: FIGMA - the title lands as punctuation AFTER the music, not before it. Big, quiet, confident. Hold it for two seconds and move. -->

# Listening to the Radio with Rust

Thomas Eckert

---

<!-- speaker_note: Keep this to thirty seconds. You are sharing a hobby, not lecturing. Anchor quote - "I'm not an expert. This is something I've been playing with for a few months. When I find something cool, my first instinct is to share it." If you drift - You are here to share something cool. That is it. -->

About Me
========

Software engineer at Redpanda Data. Background in physics.

I'm always picking up new hobbies: cycling, climbing, homelab, watercolor, and now...

<!-- pause -->

software-defined radio.

<!-- pause -->

Accessible price point. Steep first hour.

That first hour is what this talk is for.

---

<!-- speaker_note: Frame this as "how what you just heard happens", not as four abstract nouns. Say - "We're going to take that music apart, backwards. What's in the air, how a dongle catches it, how the math reads it, and then three things you can point it at." -->

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

<!-- speaker_note: Run wave-demo here. Let the animation play while you narrate the summary. Keep it to about forty seconds; this is a beat, not a section. -->

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

All the demos today read from `rtl_tcp` — including the one that was playing when you walked in.

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

<!-- speaker_note: This is the key conceptual slide, and it is the one section you must NOT trim if you are running long. Make sure the audience gets this before moving on. Frequency is rotation speed. Amplitude is distance from center. Everything we do with SDR is about measuring these two properties. -->

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

<!-- speaker_note: Run iq-demo here. Use arrow keys to change frequency and amplitude. Show how faster rotation equals higher frequency, larger circle equals higher amplitude. Let the audience see the connection. This is worth a full minute; it is the concept the rest of the talk stands on. -->

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

<!-- speaker_note: Switch to iq-print demo. Keep it brief, 5 to 10 seconds. These are the actual numbers coming off the dongle right now. Every demo that follows processes these. Then move on. -->

Let's see the real thing.

---

<!-- speaker_note: This slide replaces the two table slides. Anchor quote - "All demodulation comes back to measuring rotation speed or distance from the origin." Name two or three examples out loud rather than reading a table; the full lists are in the repo. Say - "Ships, pagers, weather satellites, garage door openers. All of them are one of these two questions." If you drift - magnitude or phase. Everything fits in one of them. -->

Many Signals, One Idea
=======================

```
IQ samples → demodulate → process
```

<!-- pause -->

Every demodulation asks one of two questions:

<!-- pause -->

**How far is the point from the origin?** → `s.norm()`
AM broadcast · ADS-B · garage remotes · weather satellites

<!-- pause -->

**How fast is the point rotating?** → `(s * prev.conj()).arg()`
FM broadcast · pagers · ship AIS · digital voice

<!-- pause -->

Magnitude or phase. That's the whole taxonomy.

---

<!-- jump_to_middle -->

<!-- speaker_note: Anchor quote - "Multiply by the conjugate of the previous sample. Take the angle. That is FM demodulation." This is the callback to the cold open - say "Let's build the thing you heard when you walked in." If you drift - One line of math. Phase change is audio. -->

Let's build the thing you heard when you walked in.

---

<!-- speaker_note: FIGMA - this slide wants to be a vertical flow diagram, one box per rate, arrows down. The three rates are the whole design. Say - "Each rate divides evenly into the one above it. 960 over 4 is 240. 240 over 5 is 48. Pick numbers that do not divide evenly and you get a slow drift between how fast you make audio and how fast the sound card eats it." If you drift - three rates, two divisions, both whole numbers. -->

FM Radio — The Pipeline
=======================

```
IQ samples          960 kHz
  ↓  low-pass, keep every 4th sample
intermediate        240 kHz
  ↓  FM demodulate
raw audio           240 kHz
  ↓  low-pass, keep every 5th sample
audio                48 kHz
  ↓  de-emphasis
speakers
```

<!-- pause -->

Three rates. Every division is a whole number.

---

<!-- speaker_note: FIGMA - show the file as a single tall column of collapsed sections, like a minimap, with the four step bands highlighted. The point is scale - the audience should see that the whole radio is smaller than they expected. Say - "This is the entire receiver. Not a sketch, not pseudocode. It compiles, it runs, and it is what was playing when you walked in." If you drift - one file, four steps, no library doing the interesting part. -->

The Whole Radio
===============

Everything that follows lives in **one file**.

```
fm-single/src/main.rs

  rates and constants        the whole design in three numbers
  Step 1  low-pass filter    pick one station out of the noise
  Step 2  FM demodulate      turn rotation into sound
  Step 3  de-emphasis        undo the transmitter's treble boost
  main                       four calls, in order
  audio plumbing             not radio; getting samples to the card
```

<!-- pause -->

No DSP library. The interesting parts are written out by hand.

---

<!-- speaker_note: FIGMA - the "antenna hears everything" idea deserves a picture; a wide spectrum with one channel boxed. Say - "The antenna does not tune. It hears every station at once, and the dongle hands you all of it. Tuning happens here, in software." Then the second idea - decimation is not just throwing data away, it is throwing away data you have proven you no longer need. FIRST CUT IF RUNNING LONG. If you drift - filter to one channel, then keep every 4th sample. -->

FM — Step 1: Filter
====================

The antenna hears every station at once. Tuning happens in software.

```rust
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
```

<!-- pause -->

960,000 samples a second becomes 240,000. Once the high frequencies
are gone, the extra samples carry no information.

---

<!-- speaker_note: FIGMA - this is THE slide of the section. Give the three lines of math room to breathe; consider animating the rotation on the complex plane beside it. Say - "FM encodes audio as the speed of rotation. So the audio is just how far the point turned between one sample and the next. Multiplying by the conjugate of the previous sample subtracts the previous angle. The angle of what is left is the rotation, and the rotation is the audio." Anchor quote - "Multiply by the conjugate of the previous sample. Take the angle. That is FM demodulation." NEVER CUT THIS SLIDE. If you drift - phase change is audio. -->

FM — Step 2: Demodulate
========================

```rust
fn process(&mut self, input: &[Iq]) -> Vec<f32> {
    input
        .iter()
        .map(|&s| {
            // s * conj(prev), written out rather than
            // hidden inside a library
            let re = s.i * self.prev.i + s.q * self.prev.q;
            let im = s.q * self.prev.i - s.i * self.prev.q;
            self.prev = s;
            im.atan2(re) * self.gain
        })
        .collect()
}
```

<!-- pause -->

The audio _is_ the rate of phase change — the **rotation speed**.

That is the whole of FM. Three lines.

---

<!-- speaker_note: FIGMA - small slide, low drama, it is the palate cleanser before the payoff. Say - "Stations boost their treble before transmitting, because hiss lives up there and a boosted signal survives it better. We undo the boost. Skip this and every station sounds harsh and thin." Mention 75 microseconds here, 50 in Europe - it is a nice detail that the constant is a different number depending on which continent you are on. SECOND CUT IF RUNNING LONG. If you drift - they boost treble, we un-boost it. -->

FM — Step 3: De-emphasis
=========================

```rust
fn process(&mut self, samples: &mut [f32]) {
    for s in samples.iter_mut() {
        self.prev += self.alpha * (*s - self.prev);
        *s = self.prev;
    }
}
```

<!-- pause -->

75 μs in North America. 50 μs in Europe.
A constant that depends on which continent you are standing on.

---

<!-- speaker_note: FIGMA - the four lines should land one at a time, and the step comments should be visually tied back to the step slides just shown (same colours). This is the "it all fits" moment. Say - "That is the receiver. Filter, demodulate, filter, de-emphasise. Everything else in the file is reading bytes and talking to the sound card." Then BRING THE AUDIO BACK - task fm-single FREQ=97.7. Second play of the cold open track, and now they know what they are listening to. Let it run under the next slide. If you drift - four calls, in order, and then you hear it again. -->

FM — The Whole Loop
===================

```rust
let tuned     = iq_filter.process(&iq);                 // step 1
let raw_audio = fm_demod.process(&tuned);               // step 2
let mut audio = audio_filter.process_real(&raw_audio);
deemphasis.process(&mut audio);                         // step 3

ring.push(&audio);                                      // speakers
```

<!-- pause -->

That's the whole FM receiver. That's what you walked in on.

---

<!-- speaker_note: This is the Rust beat, and it belongs HERE, next to the inner loop the audience just read, not in an abstract interlude earlier. Keep it to ninety seconds. This is RustConf; nobody needs selling on Rust, they want to know what the constraint actually is. Say - "2.4 million samples a second. Four hundred nanoseconds each. A GC pause doesn't make it crackle, it makes it stop." If you drift - 400 nanoseconds, no GC, and the complex math reads like math. -->

Why Rust
========

At 2.4 million samples per second, every sample gets **~400 ns** of processing time.

<!-- pause -->

- **No garbage collector** — no surprise pauses that drop samples
- **Zero-cost iterators** — the DSP pipeline compiles to tight loops
- **Fearless concurrency** — read samples on one thread, play audio on another

<!-- pause -->

| Crate | What it does |
|-------|-------------|
| `rtl-sdr-rs` | Pure Rust driver for the dongle |
| `num-complex` | `I + jQ` just works |
| `rustfft` | FFT for spectrum analysis |
| `cpal` | Cross-platform audio output |
| `ratatui` | Terminal UIs for visualizations |

<!-- pause -->

No C dependencies. GNURadio and SDR++ are more capable than any of this —
but writing it yourself is how you _understand_ it.

---

<!-- speaker_note: Anchor quote - "AM is even simpler. Just take the magnitude. sqrt of I squared plus Q squared. That is it." Keep this section moving; its whole job is the one line diff against FM. If you drift - Magnitude equals AM. One function call. -->

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

<!-- speaker_note: Switch to AM receiver demo. Tune to 119.9 MHz (CYUL main tower). Backups if it is quiet - 119.3 (north tower), 118.9 (south arrival). VERIFY ALL THREE AT THE VENUE during the 30 minute break before your session; these are from the published CYUL chart, not measured. Vertical antenna. You may hear ATC in English or French. If tower is silent for 10 or more seconds, explain that ATC is bursty and move to the next slide while waiting. -->

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

<!-- speaker_note: This is the "why should I care" slide. Aim it at the embedded and infrastructure people in the room - they have all met a drifting RTC. Do not oversell it against NTP; the point is the cases where NTP is not reachable. Keep to forty five seconds. If you drift - no network, still needs the time. -->

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

<!-- speaker_note: This is the emotional beat of the talk and it is the reason the section survives the cut. Slow down and let the date land before the last line. Anchor quote - "I wrote a decoder for a radio station that doesn't exist anymore." If you drift - CHU ran 88 years, 15 km from my desk, and went silent two months before this talk. -->

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

<!-- speaker_note: WWV is now one slide and NO LIVE DEMO by default. The spectrum demo is optional and only if you are ahead of schedule and the carrier was visible during your break test - measured 10 to 14 dB above noise on a 2 m whip, and the sidebands do NOT make it, so you will SEE a line and never hear ticks. Do not promise audio. The honest limitation is the bridge into the next section, so this slide must end on the 15 metres versus 2 metres line. -->

WWV — and Why I Can't Hear It
=============================

CHU is gone. **WWV** in Fort Collins is still transmitting. 2,900 km away.

The R820T tuner cannot go below 24 MHz, so we bypass it —
**direct sampling**, antenna straight into the ADC.

```bash
rtl_sdr -f 5000000 -s 250000 -D
```

<!-- pause -->

I can see the carrier. I can't hear the ticks.

<!-- pause -->

A quarter wavelength at 5 MHz is **15 metres**.
My antenna is 2 metres.

<!-- pause -->

Which brings us to antenna length.

---

<!-- speaker_note: Anchor quote - "An antenna resonates at a quarter of the wavelength." Do NOT explain destructive interference; state the rule and move to the swap. If you drift - Quarter wavelength. Two numbers. Show the short antenna. -->

Antenna Length
==============

An antenna works best when its length is a **quarter of the wavelength**.

At that length it resonates — electrons oscillate with maximum efficiency.

<!-- pause -->

- FM (88 MHz) → wavelength ~3.4 m → **85 cm** per side
- ADS-B (1090 MHz) → wavelength ~27 cm → **7 cm**

<!-- pause -->

This is why you don't use the same antenna for everything.

---

<!-- jump_to_middle -->

<!-- speaker_note: SWAP THE ANTENNA HERE, physically, in front of the room. Hold up the 2 m whip, then the 7 cm stub. The physical act is the slide; say almost nothing over it. "Fifteen metres is what I'd need for WWV. This is seven centimetres, and it is exactly right for what's overhead." -->

I just swapped antennas.

<!-- pause -->

This one is 7 cm.

---

<!-- jump_to_middle -->

<!-- speaker_note: Anchor quote - "Every plane in the sky is announcing itself right now." Start ADS-B decoder demo. Run adsb-decoder, then in a second terminal run task tracker (it defaults to the montreal region now; the bare binary still defaults to ottawa, and the wrong region renders an empty map with no error). Aircraft appear on the Montreal map with callsign, altitude, speed. START IT NOW and let it accumulate UNDER the code slides so the map is full by the time you come back to it. If you drift - Same math as AM, different protocol on top. -->

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

<!-- speaker_note: Come back to the map here. It has been filling in for three or four minutes and should be busy. Name one aircraft out loud - callsign, altitude, where it is going. That specificity is the payoff of the whole section. Let it sit for a few seconds in silence. -->

Every plane in the sky above us is announcing itself right now.

<!-- pause -->

With a $30 dongle and a 7 cm antenna, we can see them all.

---

<!-- speaker_note: Anchor quote - "Everything I showed you today costs about $30 and runs on any laptop." NO Q&A - you traded it for the full 40 minutes, so close by pointing people somewhere instead. Say - "I'm not doing questions from the stage, because I'd rather you came and held the thing. The hardware is on this table. I'll be here until they throw us out, and then at the reception. There's a Discord channel for this talk." Then the last line, and stop. -->

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

**The hardware is on the table at the front. Come hold it.**

I'll be here, and at the reception after.

<!-- pause -->

The airwaves are public. The signals are free.
The tools are open source.

**Go listen.**
