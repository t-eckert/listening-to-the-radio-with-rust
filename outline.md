# Listening to the Radio with Rust — Production Notes

**Duration:** 40 minutes hard ceiling, **no Q&A**, target 38 on the clock.
Per the RustConf speaker packet, breakout sessions are 40 minutes *including* up to
10 minutes of optional Q&A. Declining Q&A buys the full 40 for content.
**Venue:** RustConf 2026, Montreal, 9 September 2026
**Slot:** Wednesday 16:00, after a 30-minute break, last breakout before lightning talks.
**Goal:** Introduce technical folks to SDR in a way that enables them to go experiment themselves.

## The Slot Shapes the Talk

Three consequences, all of them already reflected in `deck/slides.md`:

1. **The 30-minute break before the session is the tech window.** The packet only promises
   breakout speakers 15 minutes at the AV table. Use the whole break at the podium to
   re-verify 97.7, 119.9, and the ADS-B feed against the venue. Nothing else in this file
   matters as much as that.
2. **A post-break room needs a payoff before it needs a syllabus.** The FM receiver runs
   silently from walk-on; the reveal is a controlled beat right after the intro ("Now
   playing: 97.7 FM"). First sound in the room is about 3 minutes in, not 21. (The
   pre-title cold open was tried and reverted 2026-08-18 — too jarring.)
3. **No Q&A, so the close is an invitation instead.** Hardware on the table, Discord channel,
   reception afterward. The reception is the same evening, which makes this slot good for it.

All delivery notes, anchor quotes, and speaker notes are in `deck/slides.md`.

---

PANIC CARD. If you lose your place, say one of these.
(1) "So, what does this mean practically?" to transition to the next demo.
(2) "Let me show you." to switch to any demo.
(3) "The key idea is..." rotation speed is frequency, distance is amplitude.
(4) "Let's go back to the IQ plane." to re-anchor on the core concept.
You know this material. You built every demo. The audience is on your side.

## How These Files Are Used

**`deck/slides.md` is the deck itself — there is no second copy.** It is a
[Slidev](https://sli.dev) presentation: markdown in, browser out, PDF for the submission.
The earlier plan to rebuild the slides in Figma is dead; Figma's motion features are
gated behind a paid feature flag, and a PDF cannot carry animation anyway.

- `deck/slides.md` is the **single source of truth** — wording, ordering, what each slide
  says, and what to say out loud. Write talking points into it, not into chat.
- Speaker notes are the **last HTML comment in each slide**. That is a Slidev rule, not a
  preference: earlier comments in the same slide are ignored, so a slide gets one merged
  note block, not several.
- Slide count and ordering are a design signal. One idea per slide; do not pack.
- Code fences take a line-step spec: ```` ```rust {all|7-8|10} ```` advances the highlight on
  each click. Use it where the point is *which line*, not just the code.

```bash
cd deck
npm run dev      # localhost:3030, presenter view at /presenter
npm run export   # radio-talk.pdf, one page per slide
```

Presenting: laptop screen shows `/presenter` (notes, timer, next slide), projector shows
the deck. Runs entirely offline once installed. The exported PDF is both the Aug 25
submission and the last-resort backup if the browser misbehaves at the venue.

---

## Equipment

Two receivers, in two places. **There is no antenna swap on stage.**

**On stage (audio demos):**
- Laptop
- RTL-SDR Blog V3 dongle
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
task am-single FREQ=119.9  # test ATC; try 119.3 and 118.9 too
(cd deck && npm run dev)   # open the deck; presenter view at /presenter

# ADS-B is on the Pi, not this laptop. Confirm it from here:
curl -s http://<pi>/healthz
curl -s http://<pi>/api/v1/aircraft | head
```

The Pi must be placed, powered, and decoding **before** the break, not during it. The
break is for the stage audio and for confirming the network path to the Pi still works
from the podium — the venue banned personal routers, so that path is conference WiFi or
the hardline Tina offered, and it is the single most fragile thing in the talk.

The receiver (`task fm-single FREQ=97.7`) starts during the break and runs muted from
walk-on. At the reveal slide, bring the fader up: "I figure we get back into things with
a little music."

A failed reveal is the worst possible failure, so have the pre-recorded IQ file loaded in
a second terminal and one keystroke away. If live RF is dead at 15:55, do the reveal from
the recording instead and say so — "this one's recorded, we'll go live later." Do not do
the reveal on silence.

## Demo Commands

| Demo | Command |
|------|---------|
| EM wave animation | `task wave` |
| IQ visualization (arrow keys) | `task iq` |
| Raw IQ from dongle | `task iq-print` |
| FM radio | `task fm FREQ=97.7` |
| FM radio, single file (the one the slides show) | `task fm-single FREQ=97.7` |
| AM aviation (the one the slides show) | `task am-single FREQ=119.9` |
| AM aviation, multi-file version | `task am FREQ=119.9` |
| ADS-B | on the Pi: `skyward run`. On the laptop: browser at `http://<pi>/` (UI not built yet). |

## Montreal Frequencies

These come from published sources, **not** from measurement at the venue. Test them the
day before; a dead demo on stage is the failure mode.

| Demo |  Montreal (new) | Notes |
|------|---|---|
| FM | **97.7 CHOM** (rock) | Backup: 95.9 Virgin |
| AM aviation |  **119.9 CYUL main tower** | Backups: 119.3 north tower, 118.9 south arrival |

ADS-B is 1090 MHz everywhere.

## Time Signals — CHU Story Only, No WWV

**CHU is dead.** The NRC shut down all three frequencies on 2026-06-22. There is no live
CHU signal and no recording of one in this repo. `demos/chu-decoder` is now dead code.

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

Re-estimated 2026-08-18 against the 50-slide deck (controlled reveal, extra AM slide,
crates at the end, map bookend + standalone closer added), using the dwell times the
speaker notes themselves prescribe.

| # | Section | Min | Cum | Payoff |
|---|---------|-----|-----|--------|
| 0 | Title + about me | 1.5 | 1.5 | |
| 1 | Three things (the promise) | 0.5 | 2.0 | |
| 2 | FM reveal — audio up, name the station | 1.0 | 3.0 | **audio** |
| 3 | Coverage map + physics pivot | 1.0 | 4.0 | |
| 4 | Physics (bobbers) + `wave-demo` | 3.5 | 7.5 | animation |
| 5 | RTL-SDR: dongle, two chips, tuner, transport | 3.0 | 10.5 | |
| 6 | I/Q + complex plane + `iq-demo` + bytes + `iq-print` | 5.5 | 16.0 | animation |
| 7 | Many signals, one idea | 1.0 | 17.0 | |
| 8 | FM: pipeline, one file, 3 steps, whole loop + audio | 5.5 | 22.5 | **audio** |
| 9 | AM: same pipeline, diff slides, live ATC | 4.0 | 26.5 | **audio** |
| 10 | Time signals: CHU story | 2.5 | 29.0 | story |
| 11 | Antenna length → why the receiver is upstairs | 2.0 | 31.0 | reveal |
| 12 | ADS-B: intro, map, code, payoff | 6.5 | 37.5 | **live data** |
| 13 | Map bookend + crates | 0.75 | 38.25 | closure |
| 14 | Getting started + "Go listen." | 1.25 | 39.5 | |

**39.5 min against a 40-minute ceiling — zero slack.** Rehearse once with a timer; unless
that run lands at 37:00 or under, take cuts 1 and 2 below pre-emptively rather than live.
TIME CHECK cues are embedded in the speaker notes at Many Signals (~16:00), the AM
pipeline (~22:30), and the ADS-B reveal (~32:30), each naming the cut to take if behind.
The variable-dwell demos are where overruns actually come from; the ATC listen is capped
at ~75 s and the map payoff at ~45 s in their notes.

Sensory payoff every 4–6 minutes: sound, animation, a map filling in, and the hardware in
the audience's hands at the close. That cadence is the actual defence against a 4 PM room,
more than any individual cut. (The old "physical object passed in front of the room" beat
died with the antenna swap; if you want a mid-talk physical beat back, pass the dongle
during the RTL-SDR section and note it in that slide.)

### Structural changes from the pre-August version

- Opens with title + intro, then a controlled FM reveal (~minute 3). The pre-title cold
  open was tried and reverted — too jarring.
- First radio payoff moved from minute 21 to minute 3.
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
- **AM slides retargeted at the new `demos/am-single`**, mirroring what the FM section does
  with `fm-single`. The two files are byte-identical outside the three steps, so the
  section's claim is now demonstrable with a diff rather than asserted. Step 2 is the real
  difference (FM needs the previous sample, AM doesn't); step 3 turned out to be the same
  three lines with one character changed — de-emphasis keeps the slow-moving part, the DC
  block subtracts it. Cost: one extra slide, +0.5 min.
- **`am-receiver` had a live audio bug**, found while building `am-single`: its default
  1,024,000 Hz cannot reach 48 kHz by integer decimation (1,024,000 / 48,000 = 21.33), and
  `128_000 / 48_000` truncated to 2, so it produced 64 kHz audio into a 48 kHz sound card —
  a 1.333x overrun, dropping a quarter of the audio continuously. Fixed: default 960 kHz,
  intermediate 240 kHz, and a guard that refuses non-dividing rates instead of truncating.
  Measured before/after with a synthesised 1 kHz AM tone.
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
