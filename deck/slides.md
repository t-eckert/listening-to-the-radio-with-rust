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
Walk up. This cover slide is already up. The FM receiver is ALREADY RUNNING but
SILENT: audio muted at the mixer or volume down. Don't touch it yet. Introduce
yourself:

"Good afternoon, my name is Thomas Eckert. I want to talk to you about a hobby
I've recently picked up using Rust to write tiny software-defined radio applications."
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
Forty-five seconds, not more. The bonafides are here to buy trust for the
physics, not to brag. Say them fast and move on. The through-line: I'm a
software engineer who trained in physics, and SDR is the rare thing that needs
both halves at once.
Stay honest about the RF part: "I've built distributed systems for years, but
radio I'm only a few months into, which is exactly why this is the talk I wish
I'd had in my first hour."
If you drift: you are here to share something cool that sits between your two
backgrounds.
-->

---

# Three Things We'll Build

<ThreeApplications class="mt-12" />

<!--
My goal with this talk is to introduce you to building applications in Rust using software defined radio as input. There is a wide range of things you can build using radio signals and Rust and they all sit on top of a common foundation you'll need to understand.
-->

---
layout: center
class: text-center
---

Now playing:

## 97.7 FM

<!--
What you are listening to is a live broadcast of 97.7 FM. This broadcast is picked up by the antenna you see on stage. It goes through the RTL-SDR dongle which produces a digitized version of the signal that I decode live on my laptop using Rust code to capture the music.
-->

---

<CoverageFlow class="mt-12" />

<!--
Here is the pipeline. The antenna picks up the electromagnetic waves in the air. These waves cause electrons in the antenna to oscillate, generating a current. The RTL-SDR dongle tunes to a frequency and transforms that current into a digital spectrum. This is rendered out as IQ signals which can be demodulated. 

The demodulation step is where we differentiate the applications. Depending on the antenna length and the demodulation code, we can get FM radio, AM radio, and aircraft tracking. Those, you will see demoed today. But there are other applications you can build with these same concepts.

Frame it as a promise, not a syllabus: "You don't need to understand every box
on this yet. By the end of the talk, you will."

FADE THE MUSIC OUT as you finish this slide. The pivot to physics on the next
slide happens in silence.
-->

---
layout: center
class: text-center
---

## Let's begin with the physics.

<!--
Let's begin with the physics.
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
Anchor quote: "Imagine a bobber in a pool of still water."
Use the bobber metaphor. Slow down here. Let the audience build the mental image.
If you drift: two bobbers. One makes waves, one receives them. Electrons up and down.
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
Run wave-demo here (task wave). Let the animation play while you narrate the
summary. Keep it to about forty seconds; this is a beat, not a section.
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

---

# Hardware: Two Chips

<TwoChips tuner="R820T2" class="mt-8" />

<v-click>

**R820T2 tuner:** selects which part of the spectrum to listen to.

</v-click>

<v-click>

**RTL2832U:** 8-bit ADC, sends digital data over USB.

</v-click>

---

# Hardware: The Tuner

<TunerShift class="mt-6" />

<!--
Say this over the diagram; don't put it on the slide.

The antenna hears everything at once: FM, AM, aviation, cell towers, Wi-Fi.
That's the top row, the whole spectrum arriving together.

The tuner's one job: slide the whole spectrum down so the station you asked for
lands at zero, where cheap hardware can sample it. Think of a radio dial: you're
not filtering the other stations out, you're sliding the window. Bottom row:
your station sits on 0, the ADC's sampling window catches it, everything else
slid away with it.

That's the whole takeaway: the tuner moves your station to zero. Mixers and
local oscillators are for the repo, not the stage — don't get pulled into them
here; IQ hasn't been introduced yet.
-->

---

# From Dongle to Your Code

There are two ways to get the raw IQ bytes into your code.

<TransportConverge class="my-10" />

Use the **`rtl_sdr`** crate to read from USB, or run **`rtl_tcp`** as a separate process to serve the IQ bytestream over TCP.

<!--
Two ways to get the raw IQ bytes into your code:
- the rtl_sdr crate reads straight from the USB dongle, in-process;
- rtl_tcp runs as a separate process and serves the IQ bytestream over TCP.
Because rtl_tcp is just a TCP interface, the dongle doesn't have to be on the
same machine as your code. (You can collect on that at the ADS-B reveal.)
-->

---
layout: center
class: text-center
---

## What does this digital signal look like?

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
Anchor quote: "The dongle gives you pairs of bytes. Each pair is a point on the
complex plane." The phase changes over time. At each instant, we project the
signal onto two axes, cosine and sine. That gives us two numbers, I and Q. That
is why they come in pairs.
If you drift: points on a plane. Speed equals frequency. Distance equals
amplitude. That is the whole game.
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
This is the key conceptual slide, and it is the one section you must NOT trim if
you are running long. Make sure the audience gets this before moving on.
Frequency is rotation speed. Amplitude is distance from center.
-->

---
layout: center
class: text-center
---

## Let's see this.

<!--
Advance to the live IQ demo on the next slide. Show how faster rotation equals a
higher frequency and a bigger circle equals a higher amplitude. Let the audience
see the connection. This is worth a full minute; it's the concept the rest of
the talk stands on.
-->

---

# I/Q in Motion

<IqDemo class="mt-4" />

<!--
The iq-demo, live in the deck. The point on the right and the height of the wave
on the left are the same instant of the signal. Faster rotation is a higher
frequency; a bigger circle is a higher amplitude. Change frequency with [ and ],
amplitude with - and =, or click the buttons. Or hit Auto-sweep to let both drift
on their own while you talk; any manual control takes back over.
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

---
layout: center
class: text-center
---

## What's coming off the dongle right now

<!--
Switch to iq-print demo (task iq-print). Keep it brief, 5 to 10 seconds. These
are the actual numbers coming off the dongle right now. Every demo that follows
processes these. Then move on — the "let's build" turn belongs to the FM slide,
two ahead.
-->

---

# Many Signals, One Idea

Every demodulation asks one of two questions.

<DemodFork class="mt-6" />

<!--
Anchor quote: "All demodulation comes back to measuring rotation speed or
distance from the origin." One IQ point, two questions: how far from the origin
(magnitude), or how fast it's turning (phase). That's the fork: amplitude
modulation down one branch, frequency modulation down the other.
Name two or three examples out loud rather than reading the list; the full tables
are in the repo. "Ships, pagers, weather satellites, garage door openers. Every
one is just one of these two questions."

TIME CHECK: the clock should read about 16:00 here. Past 18:00, take cut 1
(FM Step 1) and cut 2 (FM Step 3).
-->

---
layout: center
class: text-center
---

## Let's build an FM radio tuner

<!--
Anchor quote: "Multiply by the conjugate of the previous sample. Take the angle.
That is FM demodulation." This is the callback to the cold open.
If you drift: one line of math. Phase change is audio.
-->

---

# FM Radio: The Pipeline

<PipelineMap class="mt-2" />

<!--
Walk it in beats; the slide reveals with you.
Base: the endpoints. "We start with a firehose of IQ and end with audio the sound
card wants." Then the one scaffold sentence the deck never states anywhere else:
"the dongle hands me about a million IQ points a second, and the width of that
firehose is how much spectrum I can see at once."
Click 1 (the three steps): "Three steps get us there: filter to one station,
demodulate rotation into sound, de-emphasis to fix the treble. Those are the next
three slides."
Click 2 (the rates): "As we go we throw away samples we no longer need, so the
rate falls: 960, 240, 48 kilohertz."
Click 3 (the divisions): "And the divisions are whole numbers by design. 960 over
4 is 240, 240 over 5 is 48. Pick numbers that don't divide evenly and you get a
slow drift between how fast you make audio and how fast the sound card eats it."
The 240->48 downsample is folded into Step 3; in the code it's Step 1's low-pass
reused on the audio, with de-emphasis running at 48 kHz.
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
Show the file as a single tall column of collapsed sections, like a minimap, with
the four step bands highlighted. The point is scale: the audience should see that
the whole radio is smaller than they expected.
Say: "This is the entire receiver. Not a sketch, not pseudocode. It compiles, it
runs, and it is what was playing when you walked in."
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
The "antenna hears everything" idea deserves a picture: a wide spectrum with one
channel boxed.
Say: "The antenna does not tune. It hears every station at once, and the dongle
hands you all of it. Tuning happens here, in software." Then: decimation is not
just throwing data away, it is throwing away data you have proven you no longer need.
FIRST CUT IF RUNNING LONG.
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
This is THE slide of the section. Give the three lines of math room to breathe.
Say: "FM encodes audio as the speed of rotation. So the audio is just how far the
point turned between one sample and the next. Multiplying by the conjugate of the
previous sample subtracts the previous angle. The angle of what is left is the
rotation, and the rotation is the audio."
Anchor quote: "Multiply by the conjugate of the previous sample. Take the angle.
That is FM demodulation."
NEVER CUT THIS SLIDE.
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
Small slide, low drama, it is the palate cleanser before the payoff.
Say: "Stations boost their treble before transmitting, because hiss lives up
there and a boosted signal survives it better. We undo the boost. Skip this and
every station sounds harsh and thin."
SECOND CUT IF RUNNING LONG.
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
This is the "it all fits" moment.
Say: "That is the receiver. Filter, demodulate, filter, de-emphasise. Everything
else in the file is reading bytes and talking to the sound card."
Then BRING THE AUDIO BACK: task fm-single FREQ=97.7. Second play of the opening
reveal track, and now they know what they are listening to. Let it run under the
next slide.
-->

---

# AM Radio: The Same Pipeline

<PipelineMap class="mt-2" :steps="[
  { n: 1, name: 'Filter', sub: 'pick one station out of the noise', divide: 4 },
  { n: 2, name: 'Demodulate', sub: 'turn amplitude into sound' },
  { n: 3, name: 'DC block', sub: 'downsample, then remove the DC offset', divide: 5 },
]" />

<!--
Same map as the FM pipeline; walk it the same way, in the same beats. The only
differences are the two middle names: Step 2 reads the distance from the origin
instead of the rotation, and Step 3 blocks DC instead of de-emphasising. Same
endpoints, same three rates, same whole-number divisions.
Anchor quote: "AM is even simpler. Just take the magnitude, sqrt of I squared plus
Q squared. That's it." Keep this section moving; its whole job is the one-line
diff against FM.
Say: "Same three rates as the FM receiver. Same two divisions. I didn't pick
different numbers, because there was no reason to. Only the middle step changed."

TIME CHECK: about 22:30 here. Past 24:00, skip "Why You'd Still Want This" when
you reach the time-signal section.
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
The two code blocks should sit side by side, FM above or left, AM below or right,
with the differing line highlighted in both. This is the payoff slide of the section.
Say: "FM needs the previous sample, because rotation is a difference. AM doesn't,
because distance isn't."
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
The whole slide is one character of difference. The last click highlights only
lines 3 and 7, which is the entire point.
Say: "Step three in the FM receiver kept the slow-moving part. Step three here
subtracts it. Same tracker, same three lines, one character apart, and one is a
low-pass, the other a high-pass."
Do not over-explain the DC offset; the envelope never goes negative, speakers
want zero-centred, done.
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
START THE AM DEMO HERE: task am-single FREQ=119.9 (CYUL main tower).
Backups if it is quiet: 119.3 (north tower), 118.9 (south arrival). VERIFY ALL
THREE AT THE VENUE during the 30 minute break; these are from the published CYUL
chart, not measured. Vertical antenna, the same 2 m whip that did FM.
You may hear ATC in English or French. ATC is bursty: if the tower is silent for
10 or more seconds, say so and let it sit; a pause on a real channel is more
convincing than a recording. But cap the whole listen at about 75 seconds even
if it stays quiet — call it a quiet tower and move on. This is where overruns
come from.
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
Anchor quote: "Everything so far has been listening. This one is about knowing
what time it is." The shift: other demos turn radio into sound or data. This one
turns radio into a clock.
-->

---

# Why You'd Still Want This

Your hardware has no WiFi, no cell, no GPS.
How does it learn what time it is?

<v-clicks>

- A sensor in a mine, a basement, a ship's hull
- An air-gapped machine that will never reach an NTP server
- A real-time clock that drifts seconds a week
- GPS solves this, but it needs a view of the sky, and it's easy to jam

</v-clicks>

<v-click>

Shortwave refracts off the ionosphere. It arrives from
**thousands of kilometres away, through walls.**

</v-click>

<!--
This is the "why should I care" slide. Aim it at the embedded and infrastructure
people in the room; they have all met a drifting RTC. Do not oversell it against
NTP; the point is the cases where NTP is not reachable. Keep to forty five seconds.
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
This is the emotional beat of the talk and it is the reason the section survives
the cut. Slow down and let the date land before the last line.
Anchor quote: "I wrote a decoder for a radio station that doesn't exist anymore."
If you drift: CHU ran 88 years, 15 km from my desk, and went silent two months
before this talk.
-->

---

# Antenna Length

An antenna works best when its length is a **quarter of the wavelength**.

At that length it resonates: electrons oscillate with maximum efficiency.

<v-clicks>

- CHU (7.85 MHz) → wavelength 38 m → **9.5 m**
- FM (88 MHz) → wavelength ~3.4 m → **85 cm**
- ADS-B (1090 MHz) → wavelength ~27 cm → **7 cm**

</v-clicks>

<v-click>

Same physics, two orders of magnitude apart.
This is why you don't use the same antenna for everything.

</v-click>

<!--
THE TRANSITION IS SPOKEN, not on a slide. Let the CHU line sit in silence for a
beat first, then: "CHU was on 7.85 megahertz. Shortwave. And the length of that
wave decides everything about what you can hear."
Anchor quote: "An antenna resonates at a quarter of the wavelength."
Do NOT explain destructive interference; state the rule and let the three numbers
do the work. Gesture at the 2 m whip on stage when you say 85 cm.
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
The ADS-B intro, placed ahead of the remote-receiver slide so the audience knows
what we're about to demo before we explain where the receiver lives. This is the
third of the three we promised at the top: music, a voice, aircraft.
Say: "Two demos down. FM was music, AM was a voice on the aviation band. The
third is my favourite. Every aircraft up there is announcing itself right now:
position, altitude, speed, callsign, twice a second, in the clear. It's ADS-B."
The next slide explains why the receiver can't be in this room.
-->

---

# So the Receiver Isn't in This Room

1090 MHz wants a **7 cm** antenna and a **view of the sky**.

This whip is 2 m, and we are indoors.

<v-click>

<RemoteReceiver class="my-6" />

</v-click>

<!--
This slide replaces the old on-stage antenna swap. It does two jobs at once: it
is the payoff of the antenna rule, and it explains what the audience is about to
look at. Point at the 2 m whip on stage, then upward.
Say: "This antenna is wrong for 1090 megahertz, and this room is wrong too. So
the ADS-B receiver isn't here. There's a Raspberry Pi upstairs by a window with a
7 centimetre stub on it, and I'm going to talk to it over the network."
COLLECT THE CALLBACK from the transport slide: "Remember the socket. This is what
it was for."
Do NOT apologise for the receiver being remote; it is a consequence of the physics
you just explained.
-->

---
layout: center
class: text-center
---

## Let's see what's flying overhead.

<!--
Anchor quote: "Every plane in the sky is announcing itself right now."
skyward has been running on the Pi since well before your session, so there is
nothing to launch: open the browser at the Pi and the map is already populated.
Have the tab open and loaded BEFORE you walk on; do not type a URL on stage.
START IT NOW and leave it up in a second window so it keeps filling UNDER the
code slides.

TIME CHECK: about 32:30 here. Past 34:00, plan to name one aircraft at the
payoff instead of three, and keep the dwell short.
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
Keep the two code blocks side by side if it fits; the whole point is that they
are the same operation.
Say: "This is the AM demodulator again. It is operating on raw bytes off the
dongle instead of parsed complex numbers, because on a Pi at 2.4 million samples
a second that conversion is the expensive part. But the maths is the maths."
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
This is the "how does a pulse become a bit" slide. Walk the diagram left to
right: the preamble first, then read the four data bits off the picture: pulse
early, pulse late, early, late → 1 0 1 0.
Say: "It's the crudest possible encoding. The carrier is either on or off, with no
amplitude, no phase, none of the tricks FM and AM used. The only thing that
carries information is timing: which half of the microsecond the pulse lands in.
That's why finding the preamble matters so much. It's not data, it's a tuning
fork. Once you've locked onto that 8-microsecond pattern, you know where all 112
bit slots are."
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
The point of this slide: with 1.2 samples per half-slot, sloppy timing corrupts
the message, so the decoder is fanatical about where it samples.
Say: "We're sampling at 2.4 megahertz, so a half-microsecond half-slot is barely
one sample wide. If I computed each bit's position from the previous bit, the
rounding error would compound, and by bit 112 I'd be reading the wrong half of
the slot. So every position is computed from the start of the frame, and the error
stays flat instead of accumulating. And there's a backstop: a 24-bit CRC. If the
timing slipped, or a plane's message collided with another, the checksum fails
and we discard the whole thing. On a busy sky you throw away a lot of messages,
and that's fine. A dropped position is invisible, a wrong altitude is dangerous."
The us_to_sample helper is real; it's the floating-point µs→index map in demod.rs.
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
The point of this slide is that the four stages are swappable and scored against
each other, which is why the repo exists at all, and the headroom number makes
that concrete instead of abstract. The pipeline itself was already walked two
slides ago; don't re-walk it, go straight to the registry.
Say: "Every stage has a deliberately naive version and a registry of
alternatives, scored against each other on a golden capture. The baseline
detector pulls 517 valid messages out of one file. Swap in a smarter detector and
you get 2,403 from the same bytes. The naive pipeline works, and four-fifths of
the signal is still on the table. The repo is a scoreboard, not a finished thing."
Numbers are from skyward/fixtures/raw/golden.toml [headroom]. Do not oversell it;
land the stat and move to the payoff.
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
Come back to skyward here. It has been accumulating for three or four minutes and
should be busy. Name one aircraft OUT LOUD: callsign, altitude, where it is going.
That specificity is the payoff of the whole section and of the talk; a list of hex
codes is not a payoff. Let it sit for a few seconds in silence — then cap the
dwell at 30 to 45 seconds and move on. If running short, this is where the spare
time goes: name more aircraft.
-->

---

# The Whole Picture

<CoverageFlow class="mt-12" />

<!--
The same slide from the start of the talk, unchanged on purpose. Walk it once,
fast: waves push electrons, the dongle digitizes, IQ points, and then the fork —
rotation gave us FM, distance gave us AM and ADS-B.
Say: "This is the slide I showed you half an hour ago and asked you to take on
faith. Now every box on it is something you've watched run."
This is the close of the teaching; the two slides after it are the take-home.
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
Reference slide, not a read-aloud. It's here so people can photograph it. If you
call out any, call out num-complex (`I + jQ` just works) and cpal (audio on any
OS). Everything is pinned in the repo's Cargo files.
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
Anchor quote: "Everything I showed you today costs about $30 and runs on any laptop."
NO Q&A: you traded it for the full 40 minutes, so close by pointing people
somewhere instead.
Say: "I'm not doing questions from the stage, because I'd rather you came and
held the thing. The hardware is on this table. I'll be here until they throw us
out, and then at the reception. There's a Discord channel for this talk."
Then advance to the closer.
-->

---
layout: center
class: text-center
---

The airwaves are public. The code is open.

## Go listen.

<div class="mt-8 opacity-60 text-sm">github.com/t-eckert/listening-to-the-radio-with-rust</div>

<!--
This slide stays on the projector while people pack up, so it carries only the
closer and the repo. Say the two lines, then stop. Nothing after "Go listen."
-->
