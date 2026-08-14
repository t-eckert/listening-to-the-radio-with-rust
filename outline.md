# Listening to the Radio with Rust — Production Notes

**Duration:** 40 minutes hard ceiling, **no Q&A**, target 38 on the clock.
Per the RustConf speaker packet, breakout sessions are 40 minutes *including* up to
10 minutes of optional Q&A. Declining Q&A buys the full 40 for content.
**Venue:** RustConf 2026, Montreal, 9 September 2026
**Slot:** Wednesday 16:00, after a 30-minute break, last breakout before lightning talks.
**Goal:** Introduce technical folks to SDR in a way that enables them to go experiment themselves.

## The Slot Shapes the Talk

Three consequences, all of them already reflected in `slides.md`:

1. **The 30-minute break before the session is the tech window.** The packet only promises
   breakout speakers 15 minutes at the AV table. Use the whole break at the podium to
   re-verify 97.7, 119.9, and the ADS-B feed against the venue. Nothing else in this file
   matters as much as that.
2. **A post-break room needs a payoff before it needs a syllabus.** The talk opens cold with
   live FM audio, before the title slide. First sound in the room is under 60 seconds in,
   not 21 minutes in.
3. **No Q&A, so the close is an invitation instead.** Hardware on the table, Discord channel,
   reception afterward. The reception is the same evening, which makes this slot good for it.

All delivery notes, anchor quotes, and speaker notes are in `slides.md`.

---

## How These Files Are Used

**`slides.md` is a prototype, not the deck.** The real slides are rebuilt in Figma and
exported to run locally at the venue. So:

- `slides.md` is the **source of truth for content** — wording, ordering, what each slide
  says, and what to say out loud. Write talking points into it, not into chat.
- Speaker notes carry two things: the delivery cue (anchor quote, what to say if you drift)
  and, where useful, a `FIGMA -` line describing what the slide wants to look like.
- Slide count and ordering are a design signal. One idea per slide; do not pack.
- It still has to render in presenterm, since that is the prototyping loop. Keep the syntax
  valid: no bare double-dashes inside speaker note comments, `---` only as a slide
  separator, setext headers (title over `=====`).

---

## Equipment

- RTL-SDR Blog V4 dongle
- Long dipole antenna (FM/AM — vertical for AM)
- Short 7cm antenna (ADS-B)
- Laptop
- USB-C adapter if needed for venue display

## Pre-talk Setup

Use the full 30-minute break before the session, at the podium, not the 15 minutes the
packet asks for.

```bash
task build       # build release binaries
task alias       # create short names in demos/bin/
task rtl-tcp     # start in a background terminal
task fm FREQ=97.7   # test audio output at the venue
task am FREQ=119.9  # test ATC; try 119.3 and 118.9 too
task adsb        # confirm aircraft are decoding before you rely on it
task present     # open slides
```

**The cold open runs before you are introduced.** `task fm-single FREQ=97.7` is already
running with the volume down when the emcee starts talking. You walk on and raise the
volume — nothing to launch, nothing to type, no terminal on screen. Then kill the audio
before the title slide.

A failed cold open is the worst possible failure, so have the pre-recorded IQ file loaded
in a second terminal and one keystroke away. If live RF is dead at 15:55, open cold from the
recording instead and say so — "this one's recorded, we'll go live later." Do not open cold
on silence.

## Demo Commands

| Demo | Command |
|------|---------|
| EM wave animation | `task wave` |
| IQ visualization (arrow keys) | `task iq` |
| Raw IQ from dongle | `task iq-print` |
| FM radio | `task fm FREQ=97.7` |
| FM radio, single file (the one the slides show) | `task fm-single FREQ=97.7` |
| AM aviation | `task am FREQ=119.9` |
| WWV time signal (spectrum) | see below — needs `-D` direct sampling |
| ADS-B decoder | `task adsb` |
| Flight tracker (second terminal) | `flight-tracker --region montreal` |

## Montreal Frequencies — VERIFY AT THE VENUE

Every frequency below changed when the talk moved from Ottawa to RustConf Montreal.
These come from published sources, **not** from measurement at the venue. Test them the
day before; a dead demo on stage is the failure mode.

| Demo | Ottawa (old) | Montreal (new) | Notes |
|------|---|---|---|
| FM | 106.1 CHEZ | **97.7 CHOM** (rock) | Backup: 95.9 Virgin |
| AM aviation | 118.8 YOW tower | **119.9 CYUL main tower** | Backups: 119.3 north tower, 118.9 south arrival |
| Flight tracker | hard-coded Ottawa | `--region montreal` | Default is still `ottawa` |

The flight tracker's region is now a flag rather than a compile-time constant. It sets the
map viewport, the SQL bounds, and the drawn geography together — previously all three were
hard-coded to Ottawa, and an aircraft outside the longitude window was dropped by the query,
so the map rendered **empty with no error**. `--region ottawa|montreal`, or
`--bounds LAT_MIN,LAT_MAX,LON_MIN,LON_MAX` for any other city (no coastline drawn).

Montreal's river geometry is an approximation, not traced from a map. Aircraft positions are
real ADS-B and unaffected, but check the rivers if the shape matters to you.

## WWV Time Signal Demo — What Actually Works

**CHU is dead.** The NRC shut down all three frequencies on 2026-06-22. There is no live
CHU signal and no recording of one in this repo. `demos/chu-decoder` is now dead code.

**WWV on 5.000 MHz is the replacement, and it is a *spectrum* demo, not an audio demo.**
Measured 2026-08-10, office desk, 2 m vertical whip, direct sampling:

| what | result |
|---|---|
| WWV 5.000 MHz carrier | 10–14 dB above noise — **visible** |
| WWV modulation (100/500/600/1000 Hz tones) | **absent** — sidebands below the noise floor |

Verified end to end: the capture was demodulated through the real `am-receiver` binary and
the audio contains none of WWV's tones. Do not promise ticks or voice from the stage.

```bash
# Direct sampling. -D takes NO argument — writing "-D 2" silently breaks -n.
rtl_sdr -f 5000000 -s 250000 -D -n 2500000 wwv.iq
```

To get audio you need far more signal: roughly 9.5 m of wire (a quarter wave at 5 MHz),
not the 2 m whip. Untested as of 2026-08-10.

**The talk does not depend on this demo.** The CHU shutdown story and the 15 m / 2 m
antenna bridge both work with no hardware at all.

## Risk Mitigation

- **Pre-record IQ samples** for all demos. The code reads from a file source identically to a live source — swap one line.
- **Pre-record terminal sessions** (asciinema) as last-resort backup.
- **Build release binaries** ahead of time. Don't compile on stage.

## If a Demo Fails

Don't apologize at length. Say "Live RF — that's part of the fun." Move on.

You can always fall back to `wave-demo` and `iq-demo` — they need no hardware.

The talk works even if every live demo fails. The physics, the concepts, and the code slides carry the story.

## Timing

| # | Section | Min | Cum | Payoff |
|---|---------|-----|-----|--------|
| 0 | **Cold open** — live FM, no title slide | 1.5 | 1.5 | audio |
| 1 | Title + about me + roadmap | 1.5 | 3.0 | |
| 2 | Physics (bobbers) + `wave-demo` | 4.0 | 7.0 | animation |
| 3 | RTL-SDR: two chips, tuner, rtl_tcp | 2.5 | 9.5 | |
| 4 | I/Q + complex plane + `iq-demo` + `iq-print` | 5.5 | 15.0 | animation |
| 5 | Many signals, one idea (merged tables) | 1.0 | 16.0 | |
| 6 | FM: pipeline, one file, 3 steps, whole loop + demo | 5.0 | 21.0 | **audio** |
| 7 | Why Rust + crates | 1.5 | 22.5 | |
| 8 | AM: pipeline, one-line diff, live ATC | 3.5 | 26.0 | **audio** |
| 9 | Time signals: CHU story, WWV limitation | 3.0 | 29.0 | story |
| 10 | Antenna length + physical swap | 2.0 | 31.0 | **object** |
| 11 | ADS-B: pipeline, decode, live map | 5.5 | 36.5 | **map** |
| 12 | Close + invitation | 1.5 | 38.0 | |

**38.0 min against a 40-minute ceiling.** Two minutes of headroom, no Q&A.

Sensory payoff every 4–6 minutes: sound, animation, a physical object passed in front of
the room, a map filling in. That cadence is the actual defence against a 4 PM room, more
than any individual cut.

### Structural changes from the pre-August version
- Opens cold with FM audio; the title slide lands *after* the music.
- First radio payoff moved from minute 21 to minute 1.
- Two demodulation table slides merged into one "magnitude or phase" slide.
- "Why Rust" moved out of an abstract interlude and placed next to the FM inner loop.
- Ecosystem crate table folded into the same slide.
- "Why Not Longer?" (destructive interference) cut entirely.
- WWV reduced to one slide, no live demo by default.
- Q&A dropped; close is now hardware table + Discord + reception.

### If running long (cut in this order)
1. **FM Step 1 (filter slide).** Saves ~45 s. The pipeline slide already made the point.
2. **FM Step 3 (de-emphasis).** Saves ~45 s.
3. **"Why You'd Still Want This"** in the time-signal section. Saves ~45 s; the CHU story
   survives on its own.
4. **Shorten ADS-B accumulation** — name one aircraft instead of three.

Never cut: "Points on the Complex Plane", "FM — Step 2: Demodulate", the antenna swap.

### If running short
- Let ADS-B accumulate longer and name more aircraft.
- Deeper dive with `iq-demo` — sweep frequency and amplitude with the arrow keys.
- Optional WWV spectrum demo, but only if the carrier was visible during the break test.
