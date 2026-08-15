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

<!-- speaker_note: FIGMA - two arrows converging into one box labelled "your code". The convergence IS the point; do not draw them as separate pipelines. This slide used to be rtl_tcp trivia and now it does narrative work, so give it its beat. Say - "The dongle produces one thing: bytes. How they get to your program is a plumbing decision, not a radio decision. On this laptop it's a pipe. Across a network it's a socket. Your code can't tell the difference." Then plant the seed - "Which means the radio doesn't have to be in the same room as the code. Hold on to that one." You collect on it at the ADS-B reveal. If you drift - same bytes, two transports, one program. -->

From Dongle to Your Code
========================

The dongle produces exactly one thing: a stream of raw IQ bytes.

<!-- pause -->

```
   this laptop      rtl_sdr  ── pipe ────┐
                                         ├──▶  your code
   somewhere else   rtl_tcp  ── socket ──┘
```

<!-- pause -->

The FM receiver you walked in on reads a **pipe**.
The AM receiver you'll hear later reads a **socket**.

Neither one knows the difference.

<!-- pause -->

Which means the radio doesn't have to be in the same room as the code.

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

**How far is the point from the origin?** → `√(I² + Q²)`
AM broadcast · ADS-B · garage remotes · weather satellites

<!-- pause -->

**How fast is the point rotating?** → `the angle it turned since last time`
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

<!-- speaker_note: Anchor quote - "AM is even simpler. Just take the magnitude. sqrt of I squared plus Q squared. That is it." Keep this section moving; its whole job is the one line diff against FM. Say - "Same three rates as the FM receiver. Same two divisions. I did not pick different numbers, because there was no reason to." If you drift - Magnitude equals AM. One function call. -->

AM Radio — The Same Pipeline
============================

```
IQ samples          960 kHz
  ↓  low-pass, keep every 4th sample
intermediate        240 kHz
  ↓  AM demodulate
raw audio           240 kHz
  ↓  low-pass, keep every 5th sample
audio                48 kHz
  ↓  DC block
speakers
```

<!-- pause -->

The same three rates as the FM receiver.
Only the middle step changed.

---

<!-- speaker_note: FIGMA - the two code blocks should sit side by side, FM above or left, AM below or right, with the differing line highlighted in both. This is the payoff slide of the section. Say - "FM needs the previous sample, because rotation is a difference. AM doesn't, because distance isn't." If you drift - one needs history, the other does not. -->

AM — Step 2: Demodulate
=======================

```rust
// FM: how far did the point turn since last time?
im.atan2(re) * self.gain

// AM: how far is the point from the origin?
(s.i * s.i + s.q * s.q).sqrt()
```

<!-- pause -->

FM needs the **previous sample** — rotation is a difference.

AM needs **nothing but this sample** — distance isn't.

---

<!-- speaker_note: FIGMA - show the two structs stacked with the final line highlighted; the whole slide is one character of difference. Say - "Step three in the FM receiver kept the slow-moving part. Step three here subtracts it. Same tracker, same three lines, one character apart - and one is a low-pass, the other a high-pass." Do not over-explain the DC offset; the envelope never goes negative, speakers want zero-centred, done. If you drift - keep the slow part or subtract it. -->

AM — Step 3: DC Block
=====================

The envelope never goes negative — it rides on the carrier.
Speakers want audio centred on zero.

```rust
// FM de-emphasis: keep the slow-moving part
self.prev += self.alpha * (*s - self.prev);
*s = self.prev;

// AM DC block: subtract it
self.prev += self.alpha * (*s - self.prev);
*s -= self.prev;
```

<!-- pause -->

One tracker. Keep it and you have a low-pass.
Subtract it and you have a high-pass.

---

<!-- speaker_note: START THE AM DEMO HERE - task am-single FREQ=119.9 (CYUL main tower). Backups if it is quiet - 119.3 (north tower), 118.9 (south arrival). VERIFY ALL THREE AT THE VENUE during the 30 minute break; these are from the published CYUL chart, not measured. Vertical antenna, the same 2 m whip that did FM. You may hear ATC in English or French. ATC is bursty - if the tower is silent for 10 or more seconds, say so and let it sit; a pause on a real channel is more convincing than a recording. -->

AM — The Whole Loop
===================

```rust
let tuned     = iq_filter.process(&iq);                 // step 1
let raw_audio = am_demod.process(&tuned);               // step 2
let mut audio = audio_filter.process_real(&raw_audio);
dc_block.process(&mut audio);                           // step 3

ring.push(&audio);                                      // speakers
```

<!-- pause -->

That is the same five lines as the FM receiver,
with two words changed.

<!-- pause -->

Let's tune to 119.9 MHz — Montréal-Trudeau tower.

In Canada, receiving is legal. The law restricts transmitting
and sharing private communications, but ATC is a public broadcast.

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

<!-- speaker_note: Anchor quote - "An antenna resonates at a quarter of the wavelength." THE TRANSITION IS SPOKEN, not on a slide. Let the CHU line sit in silence for a beat first, then - "CHU was on 7.85 megahertz. Shortwave. And the length of that wave decides everything about what you can hear." Do NOT explain destructive interference; state the rule and let the three numbers do the work. Gesture at the 2 m whip on stage when you say 85 cm. If you drift - Quarter wavelength. Three frequencies, three very different lengths. -->

Antenna Length
==============

An antenna works best when its length is a **quarter of the wavelength**.

At that length it resonates — electrons oscillate with maximum efficiency.

<!-- pause -->

- CHU (7.85 MHz) → wavelength 38 m → **9.5 m**
- FM (88 MHz) → wavelength ~3.4 m → **85 cm**
- ADS-B (1090 MHz) → wavelength ~27 cm → **7 cm**

<!-- pause -->

Same physics, two orders of magnitude apart.
This is why you don't use the same antenna for everything.

---

<!-- speaker_note: This slide replaces the old on-stage antenna swap. It does two jobs at once - it is the payoff of the antenna rule, and it explains what the audience is about to look at. Point at the 2 m whip on stage, then upward. Say - "This antenna is wrong for 1090 megahertz, and this room is wrong too. So the ADS-B receiver isn't here. There's a Raspberry Pi upstairs by a window with a 7 centimetre stub on it, and I'm going to talk to it over the network." COLLECT THE CALLBACK from the transport slide - "Remember the socket. This is what it was for." Do NOT apologise for the receiver being remote; it is a consequence of the physics you just explained. -->

So the Receiver Isn't in This Room
==================================

1090 MHz wants a **7 cm** antenna and a **view of the sky**.

This whip is 2 m, and we are indoors.

<!-- pause -->

```
   7th floor, by a window          this stage
   ┌─────────────────────┐         ┌──────────────┐
   │  7 cm antenna       │         │  laptop      │
   │  RTL-SDR            │         │              │
   │  Raspberry Pi       │◄────────┤  browser     │
   │  running `skyward`  │ network │              │
   └─────────────────────┘         └──────────────┘
```

<!-- pause -->

Antenna theory is why this box is upstairs and not on the table.

---

<!-- jump_to_middle -->

<!-- speaker_note: Anchor quote - "Every plane in the sky is announcing itself right now." skyward has been running on the Pi since well before your session, so there is nothing to launch - open the browser at the Pi and the map is already populated. Have the tab open and loaded BEFORE you walk on; do not type a URL on stage. START IT NOW and leave it up in a second window so it keeps filling UNDER the code slides. If you drift - Same math as AM, different protocol on top. -->

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
  → magnitude          sqrt(I² + Q²) — the same as AM
  → preamble detect    find the 8 μs ADS-B signature
  → bit slice          112 bits from pulse positions
  → CRC-24 validate    discard what the air damaged
  → track              pair CPR frames into a position
  → HTTP               JSON and an SSE stream
```

<!-- pause -->

Four DSP stages, and only the first one is radio.

---

<!-- speaker_note: FIGMA - keep the two code blocks side by side if it fits; the whole point is that they are the same operation. Say - "This is the AM demodulator again. It is operating on raw bytes off the dongle instead of parsed complex numbers, because on a Pi at 2.4 million samples a second that conversion is the expensive part. But the maths is the maths." If you drift - magnitude is magnitude. AM and aircraft are the same measurement. -->

ADS-B — Demodulation
=====================

ADS-B is on-off keyed. Phase carries nothing,
so magnitude is the *whole* demodulator.

```rust
// rtl_sdr emits offset binary: 0..255 with 127.5 as zero.
let i = f32::from(iq[2 * k]) - 127.5;
let q = f32::from(iq[2 * k + 1]) - 127.5;
let m = (i * i + q * q).sqrt() * MAG_SCALE;
```

<!-- pause -->

The same `sqrt(I² + Q²)` you saw in the AM receiver.

Instead of audio, the pattern of high and low values encodes **bits**.

---

<!-- speaker_note: The point of this slide is that the four stages are swappable and scored against each other, which is why the repo exists at all. Say - "Every stage has a deliberately naive version and a registry of alternatives, so a new implementation lands beside the old one instead of replacing it, and a benchmark says which is better." Do not oversell it; one sentence and move to the payoff. If you drift - four stages, each one replaceable, each one measured. -->

ADS-B — Four Stages
===================

```rust
Pipeline::new(
    magnitude,   // sqrt(I² + Q²) → u16
    detector,    // where does a message start?
    slicer,      // pulse positions → 112 bits
    validator,   // CRC-24, or throw it away
)
```

<!-- pause -->

Each message is a 112-bit burst, 120 μs long — blink and you'd miss it.
At 2.4 million samples per second, we catch them all.

<!-- pause -->

Every stage has a naive baseline and a registry of alternatives,
so a faster implementation lands _beside_ the old one and gets scored.

---

<!-- jump_to_middle -->

<!-- speaker_note: Come back to skyward here. It has been accumulating for three or four minutes and should be busy. Name one aircraft OUT LOUD - callsign, altitude, where it is going. That specificity is the payoff of the whole section and of the talk; a list of hex codes is not a payoff. Let it sit for a few seconds in silence. -->

Every plane in the sky above us is announcing itself right now.

<!-- pause -->

With a $30 dongle and a 7 cm antenna, on a Pi upstairs, we can see them all.

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
