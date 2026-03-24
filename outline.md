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
| ADS-B decoder | `task adsb` |
| Flight tracker (second terminal) | `task tracker` |

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
| Antenna theory + swap | 4 |
| ADS-B demo + pipeline | 5 |
| Closing | 2 |
| **Total** | **~38 min** |

### If time is tight
- Shorten antenna theory — state the 1/4 wavelength rule without the destructive interference detail.
- Skip FM Step 1 (filter) and Step 3 (filter and play) slides — just show the demod code.

### If time is generous
- Let ADS-B run longer. Watching aircraft accumulate is satisfying.
- Deeper dive into the IQ visualization with iq-demo.
