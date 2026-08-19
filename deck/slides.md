---
theme: default
colorSchema: dark
title: Listening to the Radio with Rust
info: RustConf 2026, Montreal, 9 September 2026
author: Thomas Eckert
fonts:
  sans: Inter
transition: fade
lineNumbers: false
drawings:
  persist: false
layout: center
class: text-center
---

# Listening to the Radio with Rust

Thomas Eckert

RustConf 2026 | Montreal | 9 September 2026

<!--
[0:40 · 0:40]

[FM RECEIVER IS ALREADY RUNNING AND MUTED. Don't touch it.]

Good afternoon. My name is Thomas Eckert, and I want to talk to you about a hobby I picked up a few months ago: writing small software-defined radio applications in Rust.

***

Walk on with this slide already up. Audio muted at the mixer, not stopped — the reveal three slides from now needs the process still running.
-->

---
layout: default
class: about-me-slide
---

<div class="about-copy">

## About Me

Software engineer at **Honeycomb**. Previously at Redpanda, HashiCorp, and Microsoft. I studied Physics.

I write about what I learn at **fieldtheories.blog**.

</div>

<!-- PHOTO PLACEHOLDER: swap this div for <img src="/me.jpg" class="about-photo" />
     (drop me.jpg in deck/public/) once you've picked a photo. object-cover keeps it filling the half. -->
<div class="about-photo"></div>

<style>
.about-me-slide {
  padding: 0 !important;
  position: relative;
}
.about-me-slide .about-copy {
  width: 50%;
  height: 100%;
  padding: 3rem 3.5rem;
  display: flex;
  flex-direction: column;
  justify-content: center;
  box-sizing: border-box;
}
.about-me-slide .about-photo {
  position: absolute;
  top: 0;
  right: 0;
  width: 50%;
  height: 100%;
  object-fit: cover;
  background: linear-gradient(160deg, #38bdf8 0%, #6366f1 48%, #d946ef 100%);
}
</style>

<!--
[0:35 · 1:15]

I'm a software engineer at Honeycomb. Before that, Redpanda, HashiCorp, and
Microsoft. And before all of that I studied physics.

I write about what I learn at fieldtheories.blog.

**I've built distributed systems for years. Radio I'm only a few months into,
which is exactly why this is the talk I wish I'd had in my first hour.**
-->

---

# Three Things We'll Build

<ThreeApplications class="mt-12" />

<!--
[0:40 · 1:55]

What I want to do today is introduce you to building applications in Rust that
take radio as their input. We'll build three of them: a radio that plays music,
a radio that picks up air traffic control, and a receiver that tracks the
aircraft flying overhead.

Those three look nothing like each other. **They sit on exactly the same
foundation, and that foundation is the thing I actually want you to leave with.**
-->

---
layout: center
class: text-center
---

Now playing:

## 97.7 FM

<!--
[0:50 · 2:45]

[BRING THE AUDIO UP. Say nothing for a few seconds. Let it play.]

That is ninety-seven point seven FM. Live, right now.

There's an antenna on stage. It's picking up a broadcast from a transmitter on
Mount Royal. That goes into a USB dongle, the dongle sends my
laptop a stream of numbers, and everything after that is Rust code that I wrote.

**No radio chip, no decoder library. Numbers and arithmetic.**

[LEAVE THE MUSIC PLAYING under the next slide.]

***

If it's an ad break or dead air, name it and keep moving: "that's a commercial,
and I promise it's live." Station is CHOM 97.7, Mount Royal transmitter — from
published sources, CONFIRM AT THE VENUE during the break.
-->

---

<CoverageFlow class="mt-12" />

<!--
[1:20 · 4:05]

Here's the pipeline for processing radio signals.

The antenna picks up electromagnetic waves out of the air. Those waves push the
electrons in the metal up and down, and that motion is a current. The dongle
tunes to one slice of the spectrum and digitizes it, and what comes out the
other side is a stream of pairs of numbers. We call those IQ samples.

Everything up to that point is the same for every application. All of it. The
differences live in exactly one box, and that box is demodulation.

Change the demodulation code, and change the length of your antenna, and the
same hardware gives you music, or a controller's voice, or the position of an
aircraft. Those three are what we're building today. The list doesn't stop
there: ships, weather satellites, pagers, the tire pressure sensors in the cars.

**You don't need to understand every box on this yet. By the end of the talk,
you will.**

[FADE THE MUSIC OUT as you finish. The next slide happens in silence.]
-->

---
layout: center
class: text-center
---

## Let's begin with the physics.

<!--
[0:10 · 4:15]

Let's begin with the physics.

[PAUSE. In silence, with the music gone. Let it sit before you move on.]
-->

---

# Antenna Physics
## The Transmitter

<Bobbers class="my-6" />

Imagine a bobber sitting in a pool of still water.

<v-click>

You push it up and down. It oscillates. Waves radiate outward across the surface.

</v-click>

<v-click>

This is what happens when charged particles accelerate. Electrons moving up and
down in a wire create **electromagnetic waves** that radiate outward through space.

</v-click>

<!--
[0:55 · 5:10]

**Imagine a bobber sitting in a pool of still water.** Nothing's moving. The
surface is flat.

[click] You push it down and let it go. It oscillates. And waves spread out from
it, across the surface of the pool, in every direction.

[click] That is what happens when you accelerate a charged particle. Electrons
moving up and down in a wire make electromagnetic waves that radiate outward
through space. **That is the transmitter.**

***

SLOW DOWN. This image carries the next three slides; give them time to build it.
If you drift: two bobbers, one makes waves, one receives them. Electrons up and
down.
-->

---

# Antenna Physics
## The Receiver

<Bobbers receiver class="my-6" />

Now imagine a second bobber, sitting in the same pool some distance away.

<v-click>

The waves reach it. It begins to oscillate too, driven by the energy carried in the waves.

</v-click>

<v-click>

This is the receiving antenna. Electrons in the metal are pushed up and down by the
incoming EM wave. The antenna converts the wave back into electrical current.

</v-click>

<!--
[0:50 · 6:00]

Now put a second bobber in the same pool, some distance away.

[click] The waves reach it, and it starts to bob up and down too. The energy that moved the first bobber travelled across the pool and moved
this one.

[click] That's a receiving antenna. The incoming wave pushes electrons in the
metal up and down, and that motion is a current I can measure. **The antenna
turns the wave back into electricity.**

***

The receiver bobber is deliberately a quarter period behind the transmitter,
because the wave has to travel to reach it. Nobody will consciously notice; it
just looks right.
-->

---
layout: center
class: text-center
---

Electrons moving up and down in a *transmitting* antenna create waves.

<v-click>

Those waves *push* electrons up and down in a receiving antenna.

</v-click>

<!--
[0:45 · 6:45]

So: electrons moving up and down in a transmitting antenna make waves.

[click] And those waves push electrons up and down in a receiving antenna.

That's the whole physical link. **Everything else in this talk — every line of
Rust — is about what you do with that current once you have it.**

***

[DEMO: task wave] Optional. Let it play under the summary, forty seconds max.
This is a beat, not a section.
-->

---

# How Long Is an Antenna?

<Bobbers class="my-6" />

The distance between two crests is the **wavelength**.

<v-click>

An antenna works best when it is a **quarter of the wavelength** long.
At that length it resonates: the electrons oscillate with maximum efficiency.

</v-click>

<v-click>

97.7 FM → wavelength ~3.1 m → **77 cm** per arm

</v-click>

<v-click>

Same rule on every band. Only the number changes.

</v-click>

<!--
[0:40 · 7:25]

Look at that wave for a second. The distance between two crests is the
wavelength, and it turns out to be the number that decides everything about your
antenna.

[click] **An antenna works best when it's a quarter of the wavelength long.** At
that length it resonates, and the electrons oscillate with maximum efficiency.

[click] The station you were listening to a minute ago is at ninety-seven point
seven. That wave is about three metres long, so a quarter of it is seventy-seven
centimetres — and on a dipole like this one, that's each arm.

[POINT at the antenna on stage.] This one is a little short of that, even fully
extended. On a station this strong it makes no audible difference — but
seventy-seven centimetres is the number it's reaching for.

[click] **Same rule on every band. Only the number changes.** Hold onto that —
it decides where the third demo has to live.

***

SETUP: extend both arms fully and leave them alone. Do NOT claim it's exactly
77 cm — it isn't, and half this room owns the same antenna.
If you measure the real arm length, this slide can use the true number instead.
Do NOT explain destructive interference. State the rule and move.
This plants the callback collected on "So the Receiver Isn't in This Room."
-->

---

# Hardware: The RTL-SDR

**Software-defined radio:** digitize a chunk of spectrum, do everything else in software.

Same dongle receives FM, AM, aviation, ADS-B. Just change the frequency and the code.

<img src="/rtl-sdr.png" alt="RTL-SDR Blog V3 USB dongle" class="rtl-photo" />

<style>
.rtl-photo {
  display: block;
  margin: 2.5rem auto 0;
  width: 74%;
  max-width: 740px;
  filter: drop-shadow(0 10px 28px rgba(0, 0, 0, 0.55));
}
</style>

<!--
[0:45 · 8:10]

This is the hardware. A USB dongle, about thirty dollars. The chips inside were designed for
a television receiver and it worked
out you could ask the chip for the raw samples instead of the television
picture.

Software-defined radio means exactly this: digitize a chunk of the spectrum, and
do everything else in software. There's no FM circuit in here. There's no AM
circuit. **The same piece of plastic gives you music, a controller's voice, and
aircraft positions. You change the frequency and you change the code.**

***

It's the RTL-SDR Blog V3, with the R820T2 tuner. The V4 is discontinued — don't
recommend it from the stage.
-->

---

# Hardware: Two Chips

<TwoChips tuner="R820T2" class="mt-8" />

<v-click>

**R820T2 tuner:** selects which part of the spectrum to listen to.

</v-click>

<v-click>

**RTL2832U:** 8-bit ADC, sends digital data over USB.

</v-click>

<!--
[0:40 · 8:50]

Two chips inside it matter.

[click] The tuner, the R820T2. It decides which part of the spectrum you're
listening to. That's the dial.

[click] And the RTL2832U, which is an analog-to-digital converter. Eight bits
per sample, pushed out over USB. **Eight bits is not much, and it turns out to
be enough for everything in this talk.**
-->

---

# Hardware: The Tuner

<TunerShift class="mt-6" />

<!--
[1:00 · 9:50]

The antenna hears everything at once. FM, AM, aviation, the cell towers outside,
somebody's Wi-Fi. That's the top row: the whole spectrum, arriving together, all
the time.

The tuner has one job. It slides that entire spectrum down, so that the station
you asked for lands on zero — which is where hardware can actually sample
it. Bottom row: your station sitting on zero, the sampling window catching it,
and everything else slid away with it.

Think of turning the dial on an old radio. You are not filtering the other
stations out. **You're moving the window.**

***

Mixers and local oscillators are for the repo, not the stage. Don't get pulled
into them here — IQ hasn't been introduced yet, so you'd be spending vocabulary
you haven't earned. That's the conversation at the table afterwards.
-->

---

# From Dongle to Your Code

There are two ways to get the raw IQ bytes into your code.

<TransportConverge class="my-10" />

Use the **`rtl_sdr`** crate to read from USB, or run **`rtl_tcp`** as a separate process to serve the IQ bytestream over TCP.

<!--
[0:45 · 10:35]

Two ways to get those raw bytes into your program.

One: the rtl-sdr crate reads straight off the USB device, in your process.

Two: run rtl_tcp as a separate process, and it serves the same bytestream over a
TCP socket.

And because that second one is just a socket — **the dongle does not have to be
on the same machine as your code.**

***

This plants the ADS-B remote receiver. Collect the callback on "So the Receiver
Isn't in This Room."
-->

---
layout: center
class: text-center
---

## What does this digital signal look like?

<!--
[0:10 · 10:45]

So what does that digital signal actually look like?

***

One line, then move. The answer is the next two slides; don't preview it here.
-->

---

# I and Q

A radio signal at a single frequency is a cosine wave:

$$ signal(t) = A \cdot \cos(2\pi f t + \varphi) $$

<v-click>

The tuner shifts this to baseband. To capture both the amplitude _and_ the phase,
it samples the signal on **two axes**:

</v-click>

<v-clicks>

- **I (in-phase)**, the cosine component: `A · cos(φ)`
- **Q (quadrature)**, the sine component: `A · sin(φ)`

</v-clicks>

<v-click>

Together, `I + jQ` is a complex number that encodes both the **amplitude** and the
**phase** of the signal at each moment.

</v-click>

<!--
[1:15 · 12:00]

A radio signal at a single frequency is a cosine wave. It has an amplitude — how
strong it is — and a phase — where in its cycle it happens to be at this instant.

[click] The tuner has already slid it down to zero. Now, to capture both of
those things, the amplitude and the phase, it samples the signal on two axes
instead of one.

[click] I, for in-phase. That's the cosine component.

[click] Q, for quadrature. That's the sine component.

[click] Put the two together as a complex number, I plus j Q, and you have the
amplitude and the phase of the signal at that instant, in two numbers. **That's
why they come off the dongle in pairs.**

***

Don't defend the choice of two axes; state it and move to the picture, which is
the next slide and does the real work.
-->

---

# Points on the Complex Plane

The dongle outputs pairs of bytes: **I, Q, I, Q, ...**

Each pair is a point on the complex plane.

<v-click>

These points trace rotation around the origin.

</v-click>

<v-click>

**Speed of rotation** → proportional to the **frequency** of the wave.

</v-click>

<v-click>

**Distance from origin** → proportional to the **amplitude** of the wave.

</v-click>

<v-click>

Everything in SDR comes back to these two: how fast it turns, how far out it sits.

</v-click>

<!--
[1:10 · 13:10]

The dongle gives you pairs of bytes. I, Q, I, Q, forever. **Each pair is a point
on the complex plane.**

[click] Take those points in order, and they trace out rotation around the
origin.

[click] How fast it turns is the frequency of the wave.

[click] How far out it sits is the amplitude.

[click] And that's the whole game. **Everything in software radio comes back to
these two questions: how fast is it turning, and how far out does it sit.**

***

NEVER CUT THIS SLIDE, and never rush it. If they don't have this, nothing after
it works. Check faces before you move on.
-->


---
layout: center
class: text-center
---

## Let's see this.

<!--
[0:15 · 13:25]

Let's see that.

***

Just advance. The demo is the next slide.
-->

---

# I/Q in Motion

<IqDemo class="mt-4" />

<!--
[1:15 · 14:40]

On the left is the wave drawn the way you'd normally draw it: going up and down
over time. On the right is the same signal as a point going around a circle.
Same signal, same instant, two pictures of it.

[RAISE THE FREQUENCY: press ] a few times.]

Watch the point when I raise the frequency. It goes around faster. **That is all
a higher frequency is.**

[RAISE THE AMPLITUDE: press =.]

And when I turn the amplitude up, the circle gets bigger. The point sits further
out from the centre.

**Rotation speed is frequency. Distance from the centre is amplitude.** Every
demo after this one is arithmetic on those two facts.

***

Keys: [ and ] for frequency, - and = for amplitude, or click the buttons.
Auto-sweep drifts both while you talk; any manual key takes control back.
Worth the full minute. This is the concept the rest of the talk stands on.
-->

---

# From Bytes to IQ

The program receives IQ values as a stream of unsigned 8-bit integers. This function centers and scales these values pairwise to get the IQ values as floats.

```rust {all|4|6-9}
pub type IqSample = Complex<f32>;

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

`0` → `-1.0`. `255` → `+1.0`. The midpoint, `127.5` → `0.0`.

<!--
[0:50 · 15:30]

The dongle sends unsigned bytes, zero to two hundred and fifty-five. This is the
function that turns them into the floats we actually do arithmetic on.

[click] **Pairwise.** I, Q, I, Q — exactly as they come off the wire.

[click] Then centre and scale. Zero becomes minus one. Two fifty-five becomes
plus one. And the midpoint, a hundred and twenty-seven point five, becomes zero.

**That's the whole bridge from USB bytes to complex numbers.**

***

Don't explain `Complex<f32>` to this room.
-->

---
layout: center
class: text-center
---

## What's coming off the dongle right now

<!--
[0:25 · 15:55]

[DEMO: task iq-print. Five to ten seconds of scroll, no more.]

These are the actual numbers coming off the dongle. Right now, in this room.
**Everything you're about to see is arithmetic on this.**

***

Resist elaborating here. The "let's build" turn is two slides away and it needs
the energy more than this does.
-->

---

# Many Signals, One Idea

Every demodulation asks one of two questions.

<DemodFork class="mt-6" />

<!--
[0:45 · 16:40]

Every demodulator asks one of two questions about that
point.

How far is it from the origin? That's amplitude modulation.

Or: how fast is it turning? That's frequency modulation.

**All demodulation comes back to measuring rotation speed or distance from the
origin.** Ships, pagers, weather satellites, garage door openers, the pressure
sensor in your car tire. Every one of them is one of those two questions.

***

Name two or three out loud, don't read the list. The full tables are in the repo.

TIME CHECK: the clock should read about 16:40 as you leave this slide. Past
18:30, take the cuts marked on FM Step 1 and FM Step 3.
-->

---
layout: center
class: text-center
---

## Let's build an FM radio tuner

<!--
[0:20 · 17:00]

Let's build an FM radio tuner.

[PAUSE.]

That music you were listening to when we started — we're about to write the
thing that produced it.

***

This is the turn. Everything before it was setup; lift the energy here.
-->

---

# FM Radio: The Pipeline

<PipelineMap class="mt-2" />

<!--
[1:20 · 18:20]

Here's the shape of it. We start with a firehose of IQ samples, and we end with
audio that the sound card can play.

The dongle produces about a million IQ points a second, and the width of that
firehose is how much spectrum I can see at once.

[click] Three steps get us from one end to the other. Filter, to pick one
station out of everything else. Demodulate, to turn rotation into sound. And
de-emphasis, to fix the treble. Those are the next three slides.

[click] As we go, we throw away samples we don't need any more, so the rate
falls: nine hundred and sixty kilohertz, then two forty, then forty-eight.

[click] And those divisions are whole numbers on purpose. Nine sixty over four
is two forty. Two forty over five is forty-eight. **Pick numbers that don't
divide evenly and you get a slow drift between how fast you make audio and how
fast the sound card eats it.**

***

The 240→48 downsample is folded into Step 3 — in the code it's Step 1's low-pass
reused on the audio, with de-emphasis running at 48 kHz. Only say that if
someone asks.
-->

---

# The Whole Radio

Everything that follows lives in **one file**.

<FileMap
  class="mt-3"
  file="demos/fm-single/src/main.rs"
  :bands="[
    { label: 'header, imports', note: '', lines: 14 },
    { label: 'rates, constants', note: 'the whole design in three numbers', lines: 29 },
    { label: 'Step 1  filter', note: 'pick one station out of the noise', lines: 88, step: true },
    { label: 'Step 2  demodulate', note: 'turn rotation of IQ into sound', lines: 39, step: true },
    { label: 'Step 3  de-emphasis', note: 'undo the treble boost from the transmitter', lines: 29, step: true },
    { label: 'main', note: 'four calls, in order', lines: 70 },
    { label: 'audio plumbing', note: 'not radio; samples to the sound card', lines: 58 },
  ]"
/>

<!--
[0:50 · 19:10]

Everything I'm about to show you lives in one file, and this is that file — every
section collapsed, like a minimap, with the three steps picked out.

Header and imports. The rates and constants, which really are the whole design
in three numbers. Then the three steps. Then main, which is four calls in order.
And then audio plumbing, which isn't radio at all — it's just pushing samples to
the sound card.

**This is the entire receiver. Not a sketch, not pseudocode. It compiles, it
runs, and it is what was playing when you walked in.**

***

The point of this slide is scale — let them see it's smaller than they expected.
Don't read the band labels out; they can read.
-->

---

# FM Step 1: Filter

We have already selected a portion of the spectrum, but we need to filter out the noise from nearby stations with a low pass filter.

<SpectrumBand channel="97.7" span="960 kHz of spectrum, all at once" width="200 kHz, one station" />


```rust {all|2-4|5-12}
// A low-pass FIR filter that also decimates: keep one station, drop the rest.
// Skip the costly convolution on samples we're about to throw away.
if self.countdown >= self.decimation {
    self.countdown = 0; // fire once per `decimation` inputs, then reset
    // Convolution: each output is a weighted sum of the last n samples.
    let (mut i, mut q) = (0.0, 0.0);
    for j in 0..n {
        let h = self.history[(self.pos + j) % n]; // ring buffer of recent IQ
        i += h.i * self.taps[j]; // apply the same filter
        q += h.q * self.taps[j]; // taps to I and Q
    }
    out.push(Iq { i, q }); // emit one filtered, decimated sample
}
```

<!--
[1:00 · 20:10]

The antenna does not tune. It hears every station at once and the dongle sends
you all of it — that's the wide band on the diagram. **Tuning happens here, in
software.** A low-pass filter keeps the two hundred kilohertz that is our
station and drops everything either side of it.

[click] It also decimates in the same pass. We only do the expensive work on the
samples we're going to keep.

[click] And that expensive work is the convolution: each output is a weighted
sum of the last n samples, with the same filter taps applied to I and to Q.

**Decimation isn't throwing data away. It's throwing away data you've proven you
no longer need.**

***

FIRST CUT IF RUNNING LONG (past 18:00 at "Many Signals, One Idea").
If you cut it, say one sentence on the way past — "there's a filter first, to
pick one station out of the noise" — because Step 2 doesn't make sense without it.
-->

---

# FM Step 2: Demodulate

We get the audio out of the change in angle between each sample.

```rust {all|6-7|9|all}
fn process(&mut self, input: &[Iq]) -> Vec<f32> {
    input
        .iter()
        .map(|&s| {
            // s * conj(prev)
            let re = s.i * self.prev.i + s.q * self.prev.q;
            let im = s.q * self.prev.i - s.i * self.prev.q;
            self.prev = s;
            im.atan2(re) * self.gain
        })
        .collect()
}
```

The audio _is_ the rate of phase change, the **rotation speed**.

<!--
[1:20 · 21:30]

FM encodes the audio as the speed of rotation. So the audio is just how far the
point turned between one sample and the next.

[click] Multiplying by the conjugate of the previous sample subtracts the
previous angle. What's left is the change.

[click] Take the angle of what's left. That angle is the rotation — and the
rotation is the audio.

[click] **Multiply by the conjugate of the previous sample. Take the angle. That
is FM demodulation.**

Three lines. That's the demodulator.

***

NEVER CUT THIS SLIDE. It's the one the whole talk is pointed at — give the three
lines room to breathe and let the silence do some work.
If you drift: phase change is audio.
-->

---

# FM Step 3: De-emphasis

```rust
fn process(&mut self, samples: &mut [f32]) {
    for s in samples.iter_mut() {
        self.prev += self.alpha * (*s - self.prev);
        *s = self.prev;
    }
}
```

75 μs in North America. 50 μs in Europe.

<!--
[0:40 · 22:10]

One more step, and it's a small one.

Stations boost their treble before they transmit, because hiss lives up at the
top end and a boosted signal survives it better. So we undo the boost on the way
out. That's this line: a running average that keeps the slow-moving part.

Seventy-five microseconds in North America, fifty in Europe. **Skip it and every
station sounds harsh and thin.**

***

SECOND CUT IF RUNNING LONG.
Low drama on purpose — it's the palate cleanser before the payoff.
-->

---

# FM: The Whole Loop

This is how the loop is called within `main()`. 

```rust {all|1|2|3-4|6}
let tuned     = iq_filter.process(&iq);                 // step 1
let raw_audio = fm_demod.process(&tuned);               // step 2
let mut audio = audio_filter.process_real(&raw_audio);
deemphasis.process(&mut audio);                         // step 3

ring.push(&audio);                                      // speakers
```

<!--
[1:00 · 23:10]

And this is the loop, inside main.

[click] Filter — pick one station.

[click] Demodulate — rotation becomes sound.

[click] Filter again, then de-emphasise.

[click] And push it at the speakers.

**That is the receiver. Everything else in that file is reading bytes and
talking to the sound card.**

[BRING THE AUDIO BACK: task fm-single FREQ=97.7]

And now you know what you're listening to.

[LEAVE IT RUNNING under the next slide.]

***

Second play of the opening track — the difference is they can now name every step
that produced it. Don't talk over the first couple of seconds.

TIME CHECK: about 23:10 leaving this slide.
-->

---

# AM Radio: The Same Pipeline

<PipelineMap class="mt-2" :steps="[
  { n: 1, name: 'Filter', sub: 'pick one station out of the noise', divide: 4 },
  { n: 2, name: 'Demodulate', sub: 'turn amplitude into sound' },
  { n: 3, name: 'DC block', sub: 'downsample, then remove the DC offset', divide: 5 },
]" />

<!--
[0:50 · 24:00]

Now AM. And here's the same map — same endpoints, same three rates, same two
divisions.

[click] Step one is the same filter.

[click] Step two is where it differs. Instead of asking how fast the point is
turning, we ask how far it is from the origin.

[click] And step three blocks DC, instead of undoing a treble boost.

**I didn't pick different numbers here, because there was no reason to. Only the
middle step changed.**

***

Keep this whole section moving. Its job is the one-line diff against FM.

TIME CHECK: about 24:00 leaving this slide.
- Past 24:45 → skip "Time From the Sky" and open CHU with its line instead:
  "everything so far turned radio into sound; this one turns it into a clock."
- Past 26:45 → skip the clock section entirely. Go from the FM/AM summary
  straight to "Antenna Length." Costs you the emotional beat; buys you 1:45.
-->

---

# AM Step 2: Demodulate

```rust {1-2|4-5|all}
// FM: how far did the point turn since last time?
im.atan2(re) * self.gain

// AM: how far is the point from the origin?
(s.i * s.i + s.q * s.q).sqrt()
```

<v-click>

FM needs the **previous sample**: rotation is a difference.

AM needs **nothing but this sample**: distance isn't.

</v-click>

<!--
[0:50 · 24:50]

FM, on top. How far did the point turn since last time.

[click] AM, underneath. How far is the point from the origin. Square root of I
squared plus Q squared. **That's the whole AM demodulator.**

[click] Both of them together. One line each.

[click] And there's a real difference hiding in there. FM needs the previous
sample, because rotation is a difference. **AM needs nothing but this sample,
because distance isn't.**
-->

---

# AM Step 3: DC Block

The envelope never goes negative; it rides on the carrier.
Speakers want audio centered on zero.

```rust {1-3|5-7|3,7}
// FM de-emphasis: keep the slow-moving part
self.prev += self.alpha * (*s - self.prev);
*s = self.prev;

// AM DC block: subtract it
self.prev += self.alpha * (*s - self.prev);
*s -= self.prev;
```

<v-click>

One tracker. Keep it and you have a low-pass.
Subtract it and you have a high-pass.

</v-click>

<!--
[0:50 · 25:40]

The envelope never goes negative — it rides on top of the carrier. And speakers
want audio centred on zero. So we take the offset out.

Step three in the FM receiver kept the slow-moving part.

[click] Step three here subtracts it.

[click] And that's the entire difference. These two lines.

[click] Same tracker, same three lines. **One character apart, and one of them
is a low-pass and the other is a high-pass.**

***

Don't over-explain the DC offset. Envelope never goes negative, speakers want
zero-centred, done.
-->

---

# AM: The Whole Loop

```rust {all|2,4}
let tuned     = iq_filter.process(&iq);                 // step 1
let raw_audio = am_demod.process(&tuned);               // step 2
let mut audio = audio_filter.process_real(&raw_audio);
dc_block.process(&mut audio);                           // step 3

ring.push(&audio);                                      // speakers
```

<v-click>

That is the same five lines as the FM receiver, with two words changed.

</v-click>

<v-click>

Let's tune to 119.9 MHz, Montréal-Trudeau tower.

<span class="opacity-70">In Canada, receiving is legal. The law restricts transmitting and sharing private
communications, but ATC is a public broadcast.</span>

</v-click>

<!--
[1:45 · 27:25]

Here's the AM loop.

[click] Two words changed. am_demod instead of fm_demod. dc_block instead of
deemphasis.

[click] **That's the whole diff between a music radio and an aviation radio.**

[click] So let's point it at a hundred and nineteen point nine megahertz.
Montréal-Trudeau tower.

And before anyone worries about it: in Canada, receiving is legal. The law
restricts transmitting, and it restricts sharing private communications. Air
traffic control is a public broadcast.

[DEMO: task am-single FREQ=119.9. Then stop talking and let it run.]

***

Backups if it's quiet: 119.3 (north tower), 118.9 (south arrival). VERIFY ALL
THREE AT THE VENUE during the 30 minute break — these are from the published
CYUL chart, not measured. Same antenna that did FM, fully extended and
untouched — do NOT retune it between demos.
English or French, either is fine.
ATC is bursty. If the tower is silent for 10+ seconds, say so and let it sit — a
real pause is more convincing than a recording would be.
CAP THE LISTEN AT 75 SECONDS even if it stays quiet. Call it a quiet tower and
move on. THIS IS WHERE OVERRUNS COME FROM.
-->

---
layout: center
class: text-center
---

Both FM and AM produce audio.

<v-click>

FM uses the **speed of rotation** around the origin.

AM uses the **distance from the origin**.

</v-click>

<v-click>

Same IQ data, different interpretation.

</v-click>

<!--
[0:30 · 27:55]

So: both of those produce audio, out of the same stream of numbers.

[click] FM used the speed of rotation around the origin. AM used the distance
from the origin.

[click] Same IQ data. Different question. **That is the entire difference
between the two receivers.**

***

[FADE THE AM AUDIO OUT here if it's still running.]
-->

---

# Time From the Sky

Everything so far turned radio into **sound**.

<v-click>

This one turns radio into a **clock**.

</v-click>

<v-click>

Time signal stations broadcast the current time, continuously, straight from a
caesium atomic clock.

One-way, no network, no handshake — nothing to log into.

</v-click>

<!--
[0:35 · 28:30]

Everything we've built so far turns radio into sound.

[click] This one turns it into a clock.

[click] There are stations whose entire job is to broadcast what time it is,
continuously, straight off a caesium atomic clock. One way. No network, no
handshake, nothing to log into. **Your receiver just listens, and it knows what
time it is.**

***

Keep this short — it's the turn, not the story. The story is the next slide.
-->

---

# CHU: Ottawa, 1938–2026

Fifteen kilometres from my desk in Ottawa: **CHU**, run by the National Research Council.

3.330, 7.850, 14.670 MHz. Caesium clocks. Broadcasting since **1938**.

<v-click>

I wrote a decoder for it: Bell 103 FSK, 300 baud, BCD time code.

</v-click>

<v-click>

On **22 June 2026**, the NRC shut it off. After 88 years.

</v-click>

<v-click>

I pointed the receiver at 7.850 MHz and found **noise where a station used to be.**

</v-click>

<!--
[1:10 · 29:40]

Fifteen kilometres from my desk in Ottawa there was a station called CHU, run by
the National Research Council. Three frequencies, caesium clocks, broadcasting
since nineteen thirty-eight. It's shortwave, so it refracts off the ionosphere:
that signal comes back down thousands of kilometres away. People set their
clocks by it right across the continent.

[click] I wrote a decoder for it. Bell one-oh-three FSK, three hundred baud, a
BCD time code. And a quarter wavelength at seven point eight five megahertz is
nine and a half metres of wire, which is why nobody has a good antenna for it.

[click] On the twenty-second of June, this year, the NRC shut it off. After
eighty-eight years.

[PAUSE. Let the date land before you click again.]

[click] I pointed the receiver at seven point eight five megahertz and found
noise where a station used to be.

**I wrote a decoder for a radio station that doesn't exist any more.**

***

This is the emotional beat of the talk and it's why the section survived the cut.
Don't rush the last two clicks.
Keep the ionosphere claim about DISTANCE, not indoor reception — you measured HF
at the desk as noise-limited, so "it gets through walls" is a line you'd be
contradicting yourself on.
If you drift: CHU ran 88 years, 15 km from my desk, silent two months before
this talk.
-->

---

# ADS-B

The third demo. FM gave us music, AM gave us a voice. This one gives us **aircraft**.

<v-click>

Every plane with a transponder broadcasts its **position, altitude, speed, and
callsign** on **1090 MHz**. Twice a second. Unencrypted.

</v-click>

<v-click>

No request, no login. It's just in the air.

</v-click>

<!--
[1:00 · 30:40]

[HARD TURN. You've just come off the CHU elegy. Let one full beat of silence sit
before you speak, and drop the tone rather than bouncing straight into
enthusiasm.]

Two demos down. FM gave us music. AM gave us a voice. The third one is my
favourite, and it gives us aircraft.

[click] Every plane with a transponder is broadcasting its position, its
altitude, its speed and its callsign, on ten-ninety megahertz. Twice a second.
Unencrypted.

[click] No request, no login, no API key. **It's just in the air.**

***

This is the third of the three you promised at the top: music, a voice,
aircraft. Say it that way — the promise is being closed.
-->

---

# So the Receiver Isn't in This Room

1090 MHz wants a **7 cm** antenna and a **view of the sky**.

This one is built for the FM band, and we are indoors.

<v-click>

<RemoteReceiver class="my-6" />

</v-click>

<!--
[1:00 · 31:40]

Remember the quarter wavelength, from the very beginning.

Ten-ninety megahertz is a twenty-seven centimetre wave, so it wants a seven
centimetre antenna. And a view of the sky.

[POINT at the antenna.] This one is cut for a three metre wave — an order of
magnitude too long. And we are indoors, in a concrete building.

So the ADS-B receiver isn't in this room.

[click] There's a Raspberry Pi upstairs, next to a window, with a seven
centimetre stub on it — and I'm going to talk to it over the network.

Remember that socket, forty minutes ago. **This is what it was for.**

***

Do NOT apologise for the receiver being remote. It's a consequence of the
physics you just explained, which makes it a payoff rather than an excuse.
-->

---
layout: center
class: text-center
---

## Let's see what's flying overhead.

<!--
[0:30 · 32:10]

Let's see what's flying overhead right now.

[SWITCH TO THE ALREADY-LOADED TAB. Do not type a URL on stage.]

***

skyward has been running on the Pi since long before your session — nothing to
launch, the map is already populated. Have the tab open and loaded BEFORE you
walk on.
Leave it up in a second window so it keeps filling UNDER the code slides that
follow.

TIME CHECK: about 32:10 here. Past 33:40, name one aircraft at the payoff
instead of three, and keep the dwell short.
-->

---

# ADS-B: The Pipeline

```
IQ samples (2.4 MHz)
  → magnitude          sqrt(I² + Q²), same as AM
  → preamble detect    find the 8 μs ADS-B signature
  → bit slice          112 bits from pulse positions
  → CRC-24 validate    discard what the air damaged
  → track              pair CPR frames into a position
  → HTTP               JSON and an SSE stream
```

<v-click>

Each message is a 112-bit burst, 120 μs long.
Four DSP stages, and only the first one is radio.

</v-click>

<!--
[0:30 · 32:40]

Here's the whole thing, top to bottom.

Magnitude — which is the AM demodulator again. Find the preamble. Slice out the
bits. Check the CRC. Then track the aircraft and serve it over HTTP.

[click] Each message is a hundred and twelve bits, and the whole burst is over
in a hundred and twenty microseconds. **Four stages, and only the first one is
radio.**

***

FIRST CUT IN THIS SECTION. "ADS-B: Four Stages" covers the same ground later and
does it better. Past 32:40 here, skip straight to the demodulator slide.
-->

---

# ADS-B: Demodulation

ADS-B is on-off keyed. Phase carries nothing, so magnitude is the _whole_ demodulator.

```rust {all|4}
// rtl_sdr emits offset binary: 0..255 with 127.5 as zero.
let i = f32::from(iq[2 * k]) - 127.5;
let q = f32::from(iq[2 * k + 1]) - 127.5;
let m = (i * i + q * q).sqrt() * MAG_SCALE;
```

<v-click>

The same `sqrt(I² + Q²)` you saw in the AM receiver.

Instead of audio, the pattern of high and low values encodes **bits**.

</v-click>

<!--
[0:40 · 33:20]

ADS-B is on-off keyed. The carrier is either there or it isn't. Phase carries
nothing at all, so magnitude is the entire demodulator.

[click] **This is the AM demodulator again.** It's working on raw bytes off the
dongle instead of parsed complex numbers, because on a Pi at two point four
million samples a second that conversion is the expensive part. But the maths is
the maths.

[click] Instead of audio, the pattern of high and low values encodes bits.
-->

---

# ADS-B: Bits in Time

<PpmBits class="my-2" />

There's no volume knob and no phase to read, only _where_ in each slot the pulse
sits. Each bit is **1 µs**, split in half: a pulse in the first half is a `1`, a
pulse in the second half is a `0`. That's pulse-position modulation.

<v-click>

The 8 µs preamble is a fixed pattern. Find it and you know two things at once: a
message starts here, and exactly where every bit slot after it begins.

</v-click>

<!--
[0:55 · 34:15]

There's no volume to read here, and no phase to read. The only thing carrying
information is timing — which half of the microsecond the pulse lands in.

Each bit is one microsecond, split in half. A pulse in the first half is a one.
A pulse in the second half is a zero. That's pulse-position modulation, and it's
about the crudest encoding there is.

[WALK THE DIAGRAM left to right.] Early, late, early, late. One, zero, one, zero.

[click] The eight microsecond preamble is a fixed pattern. Find it and you know
two things at once: a message starts here, and exactly where every bit slot
after it begins. **It isn't data. It's a tuning fork.**

***

Do NOT get into CRC or sample rates here; that's the next slide.
-->

---

# ADS-B: Why Timing Is Everything

At 2.4 million samples per second, each half-slot is only about **1.2 samples** wide.

```rust {all|2-3|5-6}
// Measured from the frame start, never bit-to-bit, so rounding
// error can't pile up across 112 bits.
let bit_start_us = 8.0 + bit_idx as f64;

let early = mag[us_to_sample(bit_start_us + 0.25)]; // middle of first half
let late  = mag[us_to_sample(bit_start_us + 0.75)]; // middle of second half
bits[bit_idx] = if early > late { 1 } else { 0 };
```

<v-click>

Drift by half a slot and the last bits land in the wrong half. **CRC-24 catches
it and throws the whole 112-bit message away**. Better nothing than a wrong altitude.

</v-click>

<!--
[0:45 · 35:00]

We're sampling at two point four megahertz, so each half-slot is about one point
two samples wide. That's the whole problem with this decoder.

[click] **Every bit is measured from the start of the frame, never from the
previous bit.** Measure bit-to-bit and the rounding error compounds — by bit a
hundred and twelve you're reading the wrong half of the slot.

[click] Then sample the middle of each half. Whichever is louder is the bit.

[click] And there's a backstop: a twenty-four bit CRC. If the timing slipped, or
two aircraft talked over each other, the checksum fails and we bin the whole
message. **A dropped position is invisible. A wrong altitude is dangerous.**

***

us_to_sample is real — the floating-point µs→index map in demod.rs.
-->

---

# ADS-B: Four Stages

```rust
Pipeline::new(
    magnitude,   // sqrt(I² + Q²) → u16
    detector,    // where does a message start?
    slicer,      // pulse positions → 112 bits
    validator,   // CRC-24, or throw it away
)
```

<v-click>

Every stage has a naive baseline and a registry of alternatives, scored on the
same capture. The baseline detector finds **517** valid messages; a smarter one
finds **2,403** on the same bytes. Four-fifths of the signal is still on the table.

</v-click>

<!--
[0:40 · 35:40]

Four stages — and every one of them is swappable.

[click] Each has a deliberately naive version and a registry of alternatives,
all scored against each other on the same golden capture. The baseline detector
pulls five hundred and seventeen valid messages out of one file. Swap in a
smarter detector and you get two thousand four hundred and three, from exactly
the same bytes.

The naive pipeline works, and four fifths of the signal is still sitting on the
table. **The repo is a scoreboard, not a finished thing.**

***

Don't re-walk the pipeline; you did that already. Go straight to the registry.
Numbers are from skyward/fixtures/raw/golden.toml [headroom]. Land the stat and
move to the payoff — don't oversell it.
-->

---
layout: center
class: text-center
---

Every plane in the sky above us is announcing itself right now.

<v-click>

With a $30 dongle and a 7 cm antenna, on a Pi upstairs, we can see them all.

</v-click>

<!--
[0:45 · 36:25]

[BACK TO SKYWARD. It's been filling for four or five minutes; it should be busy.]

Every plane in the sky above us is announcing itself right now.

[NAME A REAL AIRCRAFT OUT LOUD: callsign, altitude, where it's going.]

[click] With a thirty dollar dongle and a seven centimetre antenna, on a
Raspberry Pi upstairs, we can see all of them.

[LET IT SIT. A few seconds of silence.]

***

Name an actual aircraft — callsign, altitude, destination. That specificity is
the payoff of the whole talk; a list of hex codes is not a payoff.
Cap the dwell at 30–45 seconds. If you're running SHORT, this is where the spare
time goes: name more aircraft.
-->

---

# The Whole Picture

<CoverageFlow class="mt-12" />

<!--
[0:35 · 37:00]

This is the slide I put up half an hour ago and asked you to take on faith.

Waves push electrons. The dongle digitizes. IQ points on a plane. And then the
fork — rotation gave us FM, distance gave us AM and ADS-B.

**Every box on it is now something you've watched run.**

***

Walk it once, fast. This is the close of the teaching; everything after it is
take-home.
-->

---

# The Crates

The whole talk leans on a handful of crates. Reach for these.

<div class="crates">

<div>

**Radio & DSP**
- **`rtl-sdr-rs`**: pure-Rust driver for the dongle
- **`num-complex`**: complex numbers, `I + jQ`
- **`rustfft`**: FFT for spectrum analysis

**Audio**
- **`cpal`**: cross-platform audio output
- **`hound`**: read and write WAV files

</div>

<div>

**Terminal visualizers**
- **`ratatui`** + **`crossterm`**: the IQ and wave demos

**Aircraft tracker service**
- **`axum`** + **`tokio`**: HTTP and the live SSE stream
- **`rusqlite`**: embedded SQLite store
- **`serde`**: JSON in and out

**Plumbing**
- **`anyhow`**, **`clap`**, **`signal-hook`**

</div>

</div>

<style>
.crates { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem 3rem; margin-top: 1.5rem; }
.crates ul { margin: 0.15rem 0 1.1rem; }
.crates li { margin: 0.15rem 0; }
</style>

<!--
[0:15 · 37:15]

These are the crates it all leans on. This slide exists to be photographed, not
read.

If I call out two: num-complex, because I plus jQ just works. And cpal, because
it gets audio out on any OS.

***

Reference slide. Pause long enough for cameras, then move. Everything is pinned
in the repo's Cargo files.
-->

---

# Getting Started

**Hardware**
- RTL-SDR Blog V3 (~$30)
- Dipole antenna kit (~$10)

**No hardware yet?** Try `wave-demo` and `iq-demo`; they visualize the core
concepts with no dongle needed.

**All the code is open source**
- Talk demos (FM, AM, the receivers): `github.com/t-eckert/listening-to-the-radio-with-rust`
- Aircraft tracker (the map you saw): `github.com/t-eckert/skyward`

<v-click>

**The hardware is on the table at the front. Come hold it.**

I'll be here, and at the reception after.

</v-click>

<!--
[0:35 · 37:50]

The dongle is about thirty dollars, the dipole kit about ten. **Everything I
showed you today runs on that and a laptop.**

No hardware yet? The wave demo and the IQ demo need none — those are the two
visualizations you saw. And all of it is open source.

[click] **The hardware is on the table at the front. Come and hold it.**

I'm not taking questions from the stage — I traded that time for the talk — but
I'll be here until they throw us out, and then at the reception.

***

NO Q&A. Close by pointing people somewhere rather than asking for questions,
then advance straight to the closer.
-->

---
layout: center
class: text-center
---

The airwaves are public. The code is open.

## Go listen.

<div class="mt-8 opacity-60 text-sm">github.com/t-eckert/listening-to-the-radio-with-rust</div>

<!--
[0:15 · 38:05]

The airwaves are public. The code is open.

**Go listen.**

[STOP. Nothing after that line.]

***

This slide stays up while people pack up. Say the two lines and stop — no thank
yous, no coda. Walk to the hardware table.
-->
