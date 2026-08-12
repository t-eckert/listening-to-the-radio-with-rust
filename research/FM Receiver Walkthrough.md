# FM Receiver Walkthrough -- Study Notes

A Socratic walkthrough of `demos/fm-single/src/main.rs`, the single-file FM broadcast
receiver. Written as question / answer / resolution so it can be re-used as a self-test
later: cover the resolution, answer the question, then check.

Line references are to `demos/fm-single/src/main.rs` unless stated otherwise.

---

## Open questions -- pick up here

**Q7 (unanswered).** `design_lowpass` divides every tap by the sum of the taps
(`main.rs:123-128`), and the comment says this "makes the filter pass DC at unity gain."
Why is the plain sum of the taps equal to the filter's gain at DC? Work it out from the
mechanics: feed a constant input into the multiply-accumulate loop at `main.rs:85-89` and
follow what comes out.

**Round 1 stretch (unanswered).** The IQ filter has 51 taps and a Hamming window. After
decimating to 240 kHz the new Nyquist limit is 120 kHz. Look up the approximate transition
band width of a 51-tap Hamming-windowed sinc as a fraction of the sample rate, work out
where this filter's stopband actually begins, and decide whether everything above 120 kHz
is properly suppressed before decimation. Related: why is the audio cut at 15 kHz rather
than at the 24 kHz the sample rate would allow?

---

## The map

| Lines | What |
|---|---|
| 24-35 | Rate constants and radio constants |
| 52-101 | `LowPass` -- FIR filter + decimator |
| 108-130 | `design_lowpass` -- windowed sinc |
| 143-169 | `FmDemod` -- the conjugate multiply |
| 179-198 | `DeEmphasis` -- one-pole IIR |
| 204-268 | `main` -- byte conversion and the four-call pipeline |
| 270-326 | Audio plumbing (not radio) |

The chain is 960 kHz -> 240 kHz -> 48 kHz, decimating 4x then 5x, with the two decimations
*sandwiching* the demodulator:

```
960 kHz --filter+decimate--> 240 kHz --FM demod--> 240 kHz --filter+decimate--> 48 kHz
```

---

## Q1 -- Why two decimation steps rather than one 20x step?

**Question.** Why not decimate straight from 960 kHz to 48 kHz and demodulate there?

**My answer.** Decimating before the FM demod would give a lower-fidelity signal -- more
"jumps" between the individual points on the complex plane defined by IQ values.

**Resolution.** The instinct about jumps is exactly the right mechanism. Two corrections.

*It is a cliff, not a slope.* "Lower fidelity" suggests gradual degradation like a lower
MP3 bitrate. Sampling does not work that way. Below the limit the representation is
**exact** -- 240 kHz is not a blurrier 960 kHz, it is a perfect description of everything
within +/-120 kHz. Above the limit it does not get blurry either; it becomes confidently
wrong.

*Where the jumps become fatal.* `atan2` returns an angle in -180..+180 degrees. That is the
whole story. A rotation of 350 degrees clockwise and 10 degrees counter-clockwise produce
identical numbers. So: **once rotation between samples exceeds half a turn, the measurement
wraps and the answer is wrong.**

Rotation per sample is `360 x deviation / sample_rate`, and FM swings up to +/-75 kHz
(`MAX_DEVIATION`, `main.rs:33`):

| Sample rate | Rotation per sample at full +/-75 kHz swing | |
|---|---|---|
| 960 kHz | 28 deg | lots of headroom |
| **240 kHz** | **112 deg** | fits with margin -- what the code uses |
| 150 kHz | 180 deg | exactly at the cliff edge |
| 48 kHz | 562 deg | more than a full turn -- hopeless |

At 48 kHz the point sometimes travels more than a complete revolution between samples.
Measuring a wheel's speed by glancing at it once per rotation.

**This is the wagon-wheel effect** from old Westerns -- spoked wheels appearing to turn
slowly backwards. Film samples at 24 fps; once the wheel turns more than half a spoke
spacing per frame, the eye reports the wrong rotation. Aliasing and that wagon wheel are
the same phenomenon, and the FM version is that same wheel drawn on the complex plane.
Worth stealing for the talk.

Equivalent second view: reaching 48 kHz would require filtering to +/-24 kHz first, which
destroys most of the +/-75 kHz signal before the demodulator ever sees it.

**Why the floor is ~200 kHz and not 150 kHz.** Deviation is not the whole occupied width.
Carson's rule puts an FM channel at roughly `2 x (deviation + highest audio frequency)` =
2 x (75 + 15) = 180 kHz for mono. 240 kHz clears that; 150 kHz does not.

---

## Q1a -- What step 1 is actually doing (asked as a detour, not a quiz)

**It is not filtering away noise -- it is filtering away other stations.** Tuning to 106.1
and sampling at 960 kHz does not hand you CHEZ; it hands you a 960 kHz-wide slice of the FM
dial centred on 106.1, with every other transmitter in that window present at full strength.

**Why "low"-pass when 106.1 MHz is not a low frequency.** The tuner has already shifted the
slice so the tuned frequency sits at zero. In the IQ stream, frequencies are *offsets* from
106.1 MHz and can be negative:

| Real frequency | Offset in the IQ stream |
|---|---|
| 106.5 MHz | +400 kHz |
| **106.1 MHz** | **0** |
| 105.7 MHz | -400 kHz |

So "keep only the low frequencies" means "keep only what is near zero offset" = "keep only
what is near 106.1 MHz." **A low-pass in baseband is a band-pass on the dial.** Cutting at
+/-100 kHz (`CHANNEL_HALF_WIDTH`) keeps 106.0-106.2 MHz.

**The governing rule.** With IQ sampling, the sample rate *is* the width of spectrum you can
represent: 960 kHz gives +/-480 kHz, 240 kHz gives +/-120 kHz, 48 kHz gives +/-24 kHz.

Once the filter has removed everything past +/-100 kHz, carrying 960k samples/sec to
describe a +/-100 kHz signal is waste. Dropping 3 of 4 leaves 240k/sec, covering +/-120 kHz.
The code only computes the expensive 51-tap sum for samples it keeps (`main.rs:82`), so
filtering costs a quarter of what it would at full rate.

---

## Q2 -- Why is 48 kHz suddenly fine after demodulation?

**Question.** 48 kHz was hopeless for the FM signal. After demodulation the code filters to
15 kHz and decimates to 48 kHz. What changed?

**My answer.** The output signal from demodulation is real-valued, so we are just reducing
the number of data points, not looking at an IQ sample.

**Resolution.** The first half is the key insight. The second half undersells it -- Nyquist
has not stopped applying, it changed *form*.

| Signal | Rule | At 48 kHz |
|---|---|---|
| Complex (IQ) | usable bandwidth = `fs` | +/-24 kHz, 48 kHz total |
| Real (audio) | usable bandwidth = `fs / 2` | 0-24 kHz |

A real signal cannot express which direction the point rotates -- there is no rotation, just
one number moving up and down. So it cannot tell +19 kHz from -19 kHz, and its spectrum is
forced to be symmetric: whatever sits at +f is mirrored at -f, carrying no independent
information.

Both rules are one rule:

> Each sample buys a fixed amount of information, and `fs` samples per second buys `fs` of
> spectrum. IQ spends two numbers per sample and gets `fs` of *distinct* spectrum. A real
> signal spends one number and gets `fs`, half of which is a mirror -- leaving `fs/2`.

Two numbers per sample, twice the usable bandwidth. It is conserved.

This is also why `process_real` (`main.rs:97`) can push audio through the complex filter with
an empty `Q` -- half the multiply-adds compute zero. Wasteful, correct, and it saves the file
from carrying a second filter implementation.

---

## Q3 -- Why does multiplying by the conjugate subtract angles?

**Question.** The comment claims `s * conj(prev)` subtracts the previous angle from the
current one. Why is that true?

**My answer.** (1) Magnitudes multiply and angles add. (2) The conjugate reflects across the
real axis.

**Resolution.** Both correct. Reflecting across the real axis means theta becomes **-theta** --
conjugating negates the angle. Chain them:

```
angle(s x conj(prev)) = angle(s) + angle(conj(prev))   [angles add]
                      = angle(s) - angle(prev)          [conjugate negates]
```

That is the phase change between samples -- derived rather than asserted.

**Confirming it is those two lines.** With `s = a + jb` and `prev = c + jd`:

```
s x conj(prev) = (a + jb)(c - jd)
               = ac - jad + jbc - j^2 bd
               = (ac + bd) + j(bc - ad)        [j^2 = -1]
```

| | Derived | Code (`main.rs:162-163`) |
|---|---|---|
| real | `ac + bd` | `s.i * prev.i + s.q * prev.q` |
| imag | `bc - ad` | `s.q * prev.i - s.i * prev.q` |

Exact match. Those two lines *are* the idea, which is why the file writes them out by hand
rather than calling a library.

---

## Q4 -- The magnitude is discarded. What does that buy, and what does it cost?

**Question (a).** `atan2` uses `re` and `im` only for an angle; the magnitude `|s| x |prev|`
is computed and thrown away. What practical property does that give FM?

**My answer.** You do not lose information linearly as amplitude drops with distance.

**Resolution.** Right. The mechanical reason: scaling a complex number multiplies its
magnitude and **leaves its angle untouched**. Halve the signal strength and every recovered
audio sample is identical. The information is not in the amplitude, so losing amplitude
loses nothing.

This is why FM shrugs off what wrecks AM -- lightning, motor brushes, ignition noise, all
*amplitude* disturbances that land directly in an AM output as crackle. Classic FM receivers
had a **limiter** stage that deliberately clipped the waveform flat before demodulation,
discarding amplitude in hardware. Here it is free: `atan2` never looks.

**Question (b).** When does discarding the magnitude leave nothing useful?

**My answer.** Unsure. (Asked instead: is `atan2` doing `arctan(im/re)`?)

**On that question -- yes, with one crucial addition.** `atan(im/re)` loses information
because the division destroys the individual signs:

| Point | `im/re` | `atan(im/re)` | True angle |
|---|---|---|---|
| (1, 1) | 1 | 45 deg | 45 deg |
| (-1, -1) | 1 | 45 deg | **225 deg** (i.e. -135) |

Same ratio, opposite directions. `atan2` takes `im` and `re` as **separate arguments** so it
can see both signs and resolve the quadrant, returning the full -180..+180 range. For a
demodulator measuring rotation *direction*, that is essential -- without it you could not
tell a rising frequency from a falling one.

**Resolution to (b).** With no station, `re` and `im` are not zero -- they are noise. And
`atan2(noise, noise)` returns a **uniformly random angle**: full-scale random output, every
sample.

So the demodulator does not go quiet when reception fails. **It goes loud.** That is the
harsh hiss of an empty FM channel, and why receivers need **squelch** -- a separate check on
the magnitude, the very thing the demodulator discards, to decide whether to mute.

The uncomfortable part: the demodulator cannot tell it is failing. It reports a confident
angle either way.

Measured on 2026-08-11, comparing the 106.1 capture against a supposedly-empty channel:

```
fm.wav     peak=0.2706  rms=0.0922     <- real station
noise.wav  peak=0.2263  rms=0.0615     <- mostly noise
```

Only ~3.5 dB apart. Noise demodulates to nearly the same amplitude as music, which is why
telling them apart needed the *band-energy distribution*, not the level.

This sharpens (a): FM does not fade gracefully. It stays clean while the signal beats the
noise, then collapses to full-volume hiss over a short distance -- the threshold or **cliff
effect**, the same cliff as the Nyquist one in Q1.

---

## Q5 -- Derive the demodulator gain

**Question.** `gain: sample_rate / (2.0 * PI * MAX_DEVIATION)` (`main.rs:153`). Derive it.

**My answer.** (1) Something about normalizing by dividing by 2pi. (2) Normalize by the
maximum deviation found in an FM signal.

**Resolution.** (2) is right. (1) has the `2pi` but not the reason, and is missing the other
half. **Track the units** -- they force the formula. `atan2` gives Delta-theta in radians per
sample:

```
Delta-theta      radians/sample
/ 2pi            radians/cycle    ->  cycles/sample
x fs             samples/second   ->  cycles/second = Hz
```

```
f = Delta-theta x fs / (2pi)
```

The `2pi` converts radians into whole turns; the `fs` converts per-sample into per-second.
(Same relationship as Q1's `360 x deviation / fs`, solved for the other variable.)

Then normalize so full swing reads 1.0:

```
audio = Delta-theta x fs / (2pi x MAX_DEVIATION)
                     \_________ gain _________/
```

The constant is everything that does not change sample to sample, hoisted out of the loop so
the hot path is a single multiply.

**Round-trip check** at full deviation, fs = 240 kHz:

```
Delta-theta = 2pi x 75000 / 240000 = 1.9635 rad
gain        = 240000 / (2pi x 75000) = 0.50930
audio       = 1.9635 x 0.50930 = 1.0000   OK
```

**The gain depends on `sample_rate`**, which is why `FmDemod::new` takes it (`main.rs:149`).

*Tail on the fm-receiver bug fixed 2026-08-11:* that receiver built its demodulator with
`FmDemodulator::new(75_000.0, intermediate_rate)` where `intermediate_rate` said 256 kHz
while samples actually arrived at 320 kHz. The gain was therefore low by 256/320 -- the old
code was also playing about **20% quiet**, on top of 33% slow. Derived from the code, not
measured; the WAV was deleted before anyone noticed.

---

## Q6 -- Why does FM's output noise concentrate at high audio frequencies?

**Question.** Every station applies pre-emphasis (boosting treble) and every receiver undoes
it with de-emphasis. The comment says "hiss lives up there," which is the conclusion, not the
reason. Why does noise in an FM demodulator's output concentrate at high audio frequencies?

**My answer.** At higher sonic frequencies the IQ value rotates faster around the origin
while the sampling rate stays the same, giving larger jumps in delta-theta and less
information per cycle, so noise is a bigger problem.

**Resolution.** The conclusion is right but the mechanism is not, and the misconception is
fundamental enough to be worth fixing.

**Rotation speed is set by audio amplitude, not audio frequency.**

- **How loud** the audio is -> **how far** the carrier deviates -> how fast the point rotates
- **How high-pitched** -> **how fast the deviation itself oscillates** -> how quickly the
  rotation speed changes

| | Peak rotation speed | Cycles/sec of that speed changing |
|---|---|---|
| Loud 100 Hz bass | +/-75 kHz deviation | 100 |
| Loud 10 kHz cymbal | +/-75 kHz deviation | 10,000 |

Same peak rotation speed. The cymbal does not spin the point faster; it makes the spin rate
*change* faster. A quiet cymbal deviates less than a loud bass note.

**A test that breaks the samples-per-cycle model.** That explanation predicts the problem
shrinks as the sample rate rises. But analog FM receivers, with no sampling anywhere, show
exactly the same rising-with-frequency noise. So the cause cannot be sampling density.

**The actual mechanism.** Demodulation outputs the rate of change of phase -- it
**differentiates** -- and differentiation amplifies in proportion to frequency. If noise adds
a small random phase error `e(t)`, the demodulator outputs its derivative `de/dt`. A wobble
at audio frequency `f` with amplitude `A` differentiates to amplitude `2 pi f A`:

> Flat phase noise going in becomes output noise whose amplitude climbs **linearly** with
> audio frequency.

The signal meanwhile is flat -- +/-75 kHz of deviation gives the same output level at 100 Hz
as at 15 kHz. Constant signal, rising noise, so SNR falls steadily as pitch rises. Plotted,
the noise makes a ramp: **triangular noise**.

**Why pre-emphasis wins.** The transmitter boosts treble *before* the noise is added. The
receiver's de-emphasis cuts treble back down, and because the noise was picked up in transit
and never got the boost, cutting treble attenuates noise that was never lifted while merely
restoring the signal to flat. Free SNR exactly where it was needed.

The code is a one-pole low-pass, an exponential moving average (`main.rs:193-195`):

```rust
self.prev += self.alpha * (*s - self.prev);
```

With tau = 75 us the corner sits at `1/(2 pi tau)` ~= **2.1 kHz**, and at 48 kHz,
`alpha` ~= 0.24. Skip it and everything sounds harsh -- that is the boosted treble never
taken back down. North America uses 75 us, Europe 50 us.

---

## Running list of things worth stealing for the talk

- **The wagon wheel.** Aliasing and the backwards-spinning stagecoach wheel are the same
  phenomenon; the FM version is that wheel drawn on the complex plane.
- **A low-pass in baseband is a band-pass on the dial.** The tuner has already moved your
  station to zero.
- **Two numbers per sample buys twice the bandwidth.** Complex vs real Nyquist as one
  conserved rule rather than two formulas.
- **FM does not fade, it falls off a cliff** -- and when it fails it gets *louder*, not
  quieter, because `atan2` of noise is a confident random angle.
- **The demodulator cannot tell it is failing.** Squelch exists because the one quantity that
  would reveal it -- the magnitude -- is the one the demodulator throws away.
