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

Two receivers, in two places. **There is no antenna swap on stage.**

**On stage (audio demos):**
- Laptop
- RTL-SDR Blog V4 dongle
- 2 m telescoping antenna — FM and AM, vertical for AM
- USB-C adapter if needed for venue display

**Remote (ADS-B):**
- Raspberry Pi, 7th floor by a window, hardline if the venue provides one
- RTL-SDR + 7 cm antenna
- Running `skyward` (`~/Repos/github.com/t-eckert/skyward`), reached over the
  network from the laptop

`skyward` is one binary serving HTTP: `/api/v1/aircraft`, `/api/v1/stream` (SSE),
`/healthz`. It supersedes `demos/adsb-decoder`, `demos/adsb-api`, and
`demos/flight-tracker`, none of which are used in the talk any more.

**The ADS-B view is a web UI served by `skyward`, opened in a browser on the laptop** —
a map plus an aircraft list, fed by the SSE stream. **Being built separately, in the
`skyward` repo.** Not a work item for this repo.

Requirements it has to meet, drawn from the slot rather than from taste:
- Readable from the back of a room on a projector — few, large elements.
- Survives a brief network stall without going blank; last-known state beats an error.
- Renders something sensible with zero aircraft, in case reception is bad at 16:00.
- Runs from a recorded capture identically to a live feed (`--source file:...`), so the
  laptop fallback is the same code path.

## Pre-talk Setup

Use the full 30-minute break before the session, at the podium, not the 15 minutes the
packet asks for.

```bash
task build       # build release binaries
task alias       # create short names in demos/bin/
task rtl-tcp     # start in a background terminal
task fm FREQ=97.7   # test audio output at the venue
task am FREQ=119.9  # test ATC; try 119.3 and 118.9 too
task present     # open slides

# ADS-B is on the Pi, not this laptop. Confirm it from here:
curl -s http://<pi>/healthz
curl -s http://<pi>/api/v1/aircraft | head
```

The Pi must be placed, powered, and decoding **before** the break, not during it. The
break is for the stage audio and for confirming the network path to the Pi still works
from the podium — the venue banned personal routers, so that path is conference WiFi or
the hardline Tina offered, and it is the single most fragile thing in the talk.

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
| ADS-B | on the Pi: `skyward run`. On the laptop: browser at `http://<pi>/` (UI not built yet). |

## Montreal Frequencies — VERIFY AT THE VENUE

Every frequency below changed when the talk moved from Ottawa to RustConf Montreal.
These come from published sources, **not** from measurement at the venue. Test them the
day before; a dead demo on stage is the failure mode.

| Demo | Ottawa (old) | Montreal (new) | Notes |
|------|---|---|---|
| FM | 106.1 CHEZ | **97.7 CHOM** (rock) | Backup: 95.9 Virgin |
| AM aviation | 118.8 YOW tower | **119.9 CYUL main tower** | Backups: 119.3 north tower, 118.9 south arrival |

ADS-B is 1090 MHz everywhere, so it needs no retuning for Montreal — but whatever
displays it may still carry an Ottawa-shaped viewport. Check that before the day.

## Time Signals — CHU Story Only, No WWV

**CHU is dead.** The NRC shut down all three frequencies on 2026-06-22. There is no live
CHU signal and no recording of one in this repo. `demos/chu-decoder` is now dead code.

**WWV was cut from the talk on 2026-08-14.** It is not a trim to restore if time allows —
it is gone. Three reasons, in order of weight:

1. **The physics claim on it was wrong.** The slide said the ticks were inaudible because a
   quarter wave at 5 MHz is 15 m and the whip is 2 m. The connect/disconnect test says
   otherwise: the antenna raises the noise floor 8.3 dB above the receiver's own at 5 MHz,
   so reception is *external-noise-limited* and a perfect antenna would buy **0.6 dB**.
   The whip was never the limitation; the building is. Stating the wrong cause to a
   technical audience is the actual risk, not the missing demo.
2. **No demo.** Measured 2026-08-10: the 5.000 MHz carrier sits 10–14 dB above noise and is
   visible, but the 100/500/600/1000 Hz modulation is absent — sidebands below the floor,
   verified by pushing the capture through the real `am-receiver` binary. A visible line
   with nothing to hear is a weak payoff in a 4 PM room.
3. **It cost a minute** the talk did not have, in the one section with no live output.

The direct-sampling detail (bypassing the tuner to get below 24 MHz, `rtl_sdr -D`) went with
it. Genuinely interesting, too deep for an intro-level talk, and it lived only on that slide.

**What carries the section instead:** the CHU shutdown story, which needs no hardware, and
the "Why You'd Still Want This" slide, which is the only place the talk touches the
audience's day job. The bridge into antenna length is now spoken — CHU was on 7.85 MHz,
shortwave, and a quarter wave there is 9.5 m — which puts the station from the story at the
top of the antenna ladder instead of one the audience never hears of again.

## Risk Mitigation

- **Pre-record IQ samples** for all demos. The code reads from a file source identically to a live source — swap one line. `skyward` has this built in: `--source file:fixtures/raw/golden.cu8`.
- **Pre-record terminal sessions** (asciinema) as last-resort backup.
- **Build release binaries** ahead of time. Don't compile on stage.
- **The network path to the Pi is the new single point of failure.** It did not exist in
  the old all-local setup. If the link dies mid-talk there is no recovering it from the
  stage, so the fallback has to be a `skyward` running locally on the laptop off a recorded
  capture, started before the session and left running.

## If a Demo Fails

Don't apologize at length. Say "Live RF — that's part of the fun." Move on.

You can always fall back to `wave-demo` and `iq-demo` — they need no hardware.

The talk works even if every live demo fails. The physics, the concepts, and the code slides carry the story.

The one that hurts most is ADS-B, because it is the closing payoff and it is the only demo
whose failure mode is a network problem rather than a radio problem — and "the WiFi is bad"
is a much less charming excuse on stage than "the ionosphere is bad."

## Timing

| # | Section | Min | Cum | Payoff |
|---|---------|-----|-----|--------|
| 0 | **Cold open** — live FM, no title slide | 1.5 | 1.5 | audio |
| 1 | Title + about me + roadmap | 1.5 | 3.0 | |
| 2 | Physics (bobbers) + `wave-demo` | 4.0 | 7.0 | animation |
| 3 | RTL-SDR: two chips, tuner, dongle→code transport | 2.5 | 9.5 | |
| 4 | I/Q + complex plane + `iq-demo` + `iq-print` | 5.5 | 15.0 | animation |
| 5 | Many signals, one idea (merged tables) | 1.0 | 16.0 | |
| 6 | FM: pipeline, one file, 3 steps, whole loop + demo | 5.0 | 21.0 | **audio** |
| 7 | Why Rust + crates | 1.5 | 22.5 | |
| 8 | AM: pipeline, one-line diff, live ATC | 3.5 | 26.0 | **audio** |
| 9 | Time signals: CHU story (no WWV) | 2.0 | 28.0 | story |
| 10 | Antenna length → why the receiver is upstairs | 2.0 | 30.0 | reveal |
| 11 | ADS-B: pipeline, four stages, live aircraft | 5.5 | 35.5 | **live data** |
| 12 | Close + invitation | 1.5 | 37.0 | |

**37.0 min against a 40-minute ceiling.** Three minutes of headroom, no Q&A.

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
- WWV cut entirely, taking the deck's one incorrect physics claim and the direct-sampling
  detail with it. The antenna ladder now opens on CHU's 7.85 MHz.
- Q&A dropped; close is now hardware table + Discord + reception.
- **No on-stage antenna swap.** The 2 m whip stays put; the antenna-length rule now pays
  off as an explanation of why the ADS-B receiver is a Pi upstairs instead of a second
  antenna on the table. The physics does the same narrative work and the setup gets
  explained for free.
- ADS-B code slides rewritten against `skyward` (magnitude → detect → slice → validate)
  instead of the retired `demos/adsb-decoder`.
- The `rtl_tcp` slide became "From Dongle to Your Code" — pipe vs socket, converging on the
  same program. It used to be a stray implementation detail *and* it was inaccurate: it
  claimed every demo read from `rtl_tcp`, but `fm-single` reads raw IQ on **stdin**, piped
  from `rtl_sdr` (`Taskfile.yml`), while `fm-receiver` and `am-receiver` default to
  `tcp:127.0.0.1:1234`. The slide now states that difference instead of papering over it,
  and plants the socket idea that the ADS-B reveal collects on.

### If running long (cut in this order)
1. **FM Step 1 (filter slide).** Saves ~45 s. The pipeline slide already made the point.
2. **FM Step 3 (de-emphasis).** Saves ~45 s.
3. **"Why You'd Still Want This"** in the time-signal section. Saves ~45 s; the CHU story
   survives on its own.
4. **Shorten ADS-B accumulation** — name one aircraft instead of three.

Never cut: "Points on the Complex Plane", "FM — Step 2: Demodulate", "So the Receiver Isn't
in This Room" (it is the only thing that explains the remote Pi).

### If running short
- Let ADS-B accumulate longer and name more aircraft.
- Deeper dive with `iq-demo` — sweep frequency and amplitude with the arrow keys.
- Linger on the CHU beat. Let the 88-years line and the date sit longer than feels natural.
