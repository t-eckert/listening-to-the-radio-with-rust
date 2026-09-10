# Listening to the Radio with Rust

An introduction to software-defined radio (SDR) through Rust. This repo contains everything from a talk of the same name — demos, slides, and research — and is designed to help you get started with SDR yourself.

You don't need any prior experience with radio or signal processing. All you need is a $30 USB dongle, an antenna, and curiosity.

## Hardware

You need one thing: an **RTL-SDR Blog V3** dongle (~$30) and the **dipole antenna kit** (~$10) that comes with it.

- [RTL-SDR Blog dongles](https://www.rtl-sdr.com/buy-rtl-sdr-dvb-t-dongles/) — USB dongle
- The kit includes telescoping dipole antennas for FM/AM and a short antenna for ADS-B

That's it. Plug the dongle into your laptop, attach an antenna, and you're ready.

Everything in this repo was developed and measured against a **V3**, which uses the
R820T2 tuner. A V4 uses a different tuner (the R828D); the demos should still work, but
none of the gain figures quoted here were measured on one.

## Quick Start

```bash
# Clone the repo
git clone https://github.com/t-eckert/listening-to-the-radio-with-rust.git
cd listening-to-the-radio-with-rust/demos

# Start the RTL-SDR TCP server (in a separate terminal)
rtl_tcp

# Listen to FM radio (replace 100.3 with a local station's MHz)
cargo run -p fm-receiver -- --frequency 100.3
```

You should hear a local FM station through your speakers.

**If you get static:** the station you picked probably isn't strong where you're sitting.
Don't start swapping hardware — find out what actually reaches you first:

```bash
cargo run -p freq-scanner        # sweeps 88-108 MHz and ranks what it finds
```

A published frequency for your city is a starting guess, not a measurement. Reception
changes with the room, the window, and which side of the building you're on.

## Demos

All demos live in the `demos/` directory. Each one has its own README with details on how it works.

| Demo | What it does | Needs hardware? |
|------|-------------|----------------|
| [wave-demo](demos/wave-demo/) | Visualizes EM wave propagation between antennas | No |
| [iq-demo](demos/iq-demo/) | Interactive IQ point rotation on the complex plane | No |
| [iq-print](demos/iq-print/) | Prints raw IQ samples from the SDR | Yes |
| [fm-receiver](demos/fm-receiver/) | Demodulates and plays FM radio | Yes |
| [fm-single](demos/fm-single/) | **The whole FM receiver in one file**, reading IQ on stdin | Yes (or a recording) |
| [am-receiver](demos/am-receiver/) | Demodulates and plays AM radio | Yes |
| [am-single](demos/am-single/) | The same, for AM — diff it against `fm-single` | Yes (or a recording) |
| [freq-scanner](demos/freq-scanner/) | Sweeps a band and ranks what's actually receivable | Yes |
| [iq-play](demos/iq-play/) | Replays a recorded `.iq` file at capture rate, looping | No |
| [iq-tcp](demos/iq-tcp/) | Streams IQ from a remote `rtl_tcp` in `rtl_sdr` format | Yes (remote) |
| [chu-decoder](demos/chu-decoder/) | Decodes the CHU shortwave time signal — **station is off air**, see its README | No longer possible |
| [adsb-decoder](demos/adsb-decoder/) | Decodes aircraft positions from ADS-B signals | Yes |
| [flight-tracker](demos/flight-tracker/) | TUI map showing aircraft tracked by adsb-decoder | No (reads from adsb-decoder's database) |
| [adsb-api](demos/adsb-api/) | Serves tracked aircraft over HTTP as JSON | No (reads from adsb-decoder's database) |

`demos/sdr/` is not a demo — it's the shared library the multi-file demos build on
(`dsp/` for filtering and demodulation, `source/` for reading from a file, a TCP server,
or USB).

**Start here:** If you don't have hardware yet, try `wave-demo` and `iq-demo` — they
visualize the core concepts with no dongle needed. If you have a dongle and want to read
exactly one file that does the whole job, read [`fm-single`](demos/fm-single/).

## The Talk

This repo accompanies a talk given at:

- **RustConf** — Montreal, 9 September 2026
- **Ottawa Systems** — 24 March 2026

The slides are built with [Slidev](https://sli.dev). To run the deck locally:

```bash
cd deck
npm install
npm run dev      # opens the presentation in your browser
npm run export   # writes radio-talk.pdf
```

- [deck/slides.md](deck/slides.md) — The slide deck, with the spoken script in the speaker notes
- [outline.md](outline.md) — Talk structure

## Learn More

The `research/` directory contains deeper dives into the topics covered in the talk:

- [SDR Fundamentals](research/SDR%20Fundamentals.md) — IQ sampling, demodulation, filtering, and spectrum analysis
- [RTL SDR Hardware](research/RTL%20SDR%20Hardware.md) — How the RTL-SDR dongle works, chip architecture, and direct sampling
- [How the Demos Work](research/How%20the%20Demos%20Work.md) — Signal processing chains for each demo
- [Rust Crates](research/Rust%20Crates.md) — Evaluation of the Rust SDR ecosystem
- [CHU Time Signal](research/CHU%20Time%20Signal.md) — Canada's shortwave atomic clock signal

## Key Rust Crates

These are the main crates used across the demos:

- [`rtl-sdr-rs`](https://github.com/ccostes/rtl-sdr-rs) — Pure Rust RTL-SDR driver
- [`num-complex`](https://docs.rs/num-complex) — Complex number types for IQ samples
- [`rustfft`](https://docs.rs/rustfft) — FFT for spectrum analysis
- [`cpal`](https://docs.rs/cpal) — Cross-platform audio output
- [`ratatui`](https://docs.rs/ratatui) — Terminal UI for visualizations
