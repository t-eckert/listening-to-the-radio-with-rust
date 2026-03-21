# Listening to the Radio with Rust

The repository contains the research, demos, and presentation for "Listening to the Radio with Rust", an introduction to software-defined radio through the lens of Rust.

## Presentations

This is a ~40 minute talk being prepared for two venues:

- **Ottawa Systems** — Systems programming meetup talk on 24 March 2026
- (Potentially) **RustConf 2026** — September, Montrea

Slides are built with [presenterm](https://github.com/mfontanini/presenterm).

## Demos

Three Rust programs, each progressively more complex:

1. **fm-receiver** — Connect to an RTL-SDR, read IQ samples, demodulate FM, and play audio.
2. **chu-decoder** — Tune to the CHU Ottawa shortwave time signal and decode the time.
3. **freq-scanner** — Sweep frequencies, measure signal strength, and log active signals.

## Hardware

- RTL-SDR Blog V4 USB dongle (~$30)
- Dipole antenna kit (telescopic elements for VHF/UHF)

## Key Rust Crates

- [`rtl-sdr-rs`](https://github.com/ccostes/rtl-sdr-rs) — RTL-SDR driver
- [`rtlsdr_mt`](https://lib.rs/crates/rtlsdr_mt) — Multithreaded RTL-SDR interface
- [`demod_fm`](https://docs.rs/demod_fm/latest/demod_fm/) — FM demodulation
- [`rust_radio`](https://github.com/johnwstanford/rust_radio) — DSP library for SDR
- [`dasp`](https://github.com/RustAudio/dasp) — Digital audio signal processing
- [`soapysdr`](https://lib.rs/crates/soapysdr) — Hardware abstraction layer
