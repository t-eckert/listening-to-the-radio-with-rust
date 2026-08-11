# Listening to the Radio with Rust — Production Notes

**Duration:** 35–40 minutes
**Venue:** Bayview Yards (7 Bayview Station Rd), 2nd floor, Ottawa Systems, 24 March 2026
**Goal:** Introduce technical folks to SDR in a way that enables them to go experiment themselves.

All delivery notes, anchor quotes, and speaker notes are in `slides.md`.

---

## Equipment

- RTL-SDR Blog V4 dongle
- Long dipole antenna (FM/AM — vertical for AM)
- Short 7cm antenna (ADS-B)
- Laptop
- USB-C adapter if needed for venue display

## Pre-talk Setup

```bash
task build       # build release binaries
task alias       # create short names in demos/bin/
task rtl-tcp     # start in a background terminal
task fm FREQ=106.1  # test audio output at the venue
task present     # open slides
```

## Demo Commands

| Demo | Command |
|------|---------|
| EM wave animation | `task wave` |
| IQ visualization (arrow keys) | `task iq` |
| Raw IQ from dongle | `task iq-print` |
| FM radio | `task fm FREQ=106.1` |
| AM aviation | `task am FREQ=118.8` |
| WWV time signal (spectrum) | see below — needs `-D` direct sampling |
| ADS-B decoder | `task adsb` |
| Flight tracker (second terminal) | `task tracker` |

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

| Section | Minutes |
|---------|---------|
| Intro + roadmap | 2 |
| Physics (EM waves) | 5 |
| RTL-SDR + tuner + rtl_tcp | 3 |
| IQ samples + demos | 5 |
| Demodulation tables | 3 |
| Why Rust + ecosystem | 3 |
| FM demo + pipeline | 3 |
| AM demo + pipeline | 3 |
| Time signals (CHU story + WWV) | 4 |
| Antenna theory + swap | 4 |
| ADS-B demo + pipeline | 5 |
| Closing | 2 |
| **Total** | **~42 min** |

Runs long for a 40-minute slot on purpose — carry the surplus and cut from a real
run-through rather than trimming on paper. The cut list below is pre-decided so the
decision is cheap when the time comes.

### If time is tight
- **Drop the WWV demo, keep the CHU story.** The shutdown beat and the "15 metres vs
  2 metres" bridge into antenna length are the parts that earn their time; the spectrum
  demo is the cuttable half. Saves ~2 min and removes the only live-RF risk in the section.
- Shorten antenna theory — state the 1/4 wavelength rule without the destructive interference detail.
- Skip FM Step 1 (filter) and Step 3 (filter and play) slides — just show the demod code.

### If time is generous
- Let ADS-B run longer. Watching aircraft accumulate is satisfying.
- Deeper dive into the IQ visualization with iq-demo.
