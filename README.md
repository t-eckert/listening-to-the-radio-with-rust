# Listening to the Radio with Rust

Talks, demos, and research for "Listening to the Radio with Rust" — an introduction to software-defined radio through the lens of Rust.

## Presentations

This is a 40 minute talk being prepared for two venues:

- **Ottawa Systems** — Systems programming meetup talk on 24 March 2026
- (Potentially) **RustConf 2026** — September, Montrea

Slides are built with [presenterm](https://github.com/mfontanini/presenterm).

## Demos

Three Rust programs, each progressively more complex:

1. **fm-receiver** — Connect to an RTL-SDR, read IQ samples, demodulate FM, and play audio.
2. **chu-decoder** — Tune to the CHU Ottawa shortwave time signal and decode the time.
3. **freq-scanner** — Sweep frequencies, measure signal strength, and log active signals.

## Structure

```
slides/
  rustconf/          # 40-minute RustConf version
  ottawa-systems/    # Ottawa Systems meetup version
demos/
  fm-receiver/       # Demo 1: FM radio demodulation
  chu-decoder/       # Demo 2: CHU time signal decoder
  freq-scanner/      # Demo 3: Frequency scanner
research/            # Notes on SDR concepts, DSP, protocols
samples/             # Pre-recorded IQ sample files for offline demos
assets/              # Images, diagrams for slides
```

## Hardware

- RTL-SDR Blog V4 USB dongle (~$30)
- Dipole antenna kit (telescopic elements for VHF/UHF)
- Raspberry Pi 4 B running OpenWebRX+ (optional, for remote SDR access)

## Key Rust Crates

- [`rtl-sdr-rs`](https://github.com/ccostes/rtl-sdr-rs) — RTL-SDR driver
- [`rtlsdr_mt`](https://lib.rs/crates/rtlsdr_mt) — Multithreaded RTL-SDR interface
- [`demod_fm`](https://docs.rs/demod_fm/latest/demod_fm/) — FM demodulation
- [`rust_radio`](https://github.com/johnwstanford/rust_radio) — DSP library for SDR
- [`dasp`](https://github.com/RustAudio/dasp) — Digital audio signal processing
- [`soapysdr`](https://lib.rs/crates/soapysdr) — Hardware abstraction layer
