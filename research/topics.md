# Research Topics

Things to research and document before building demos and writing slides.

## SDR Fundamentals
- [x] IQ sampling — clear explanation with diagrams
- [x] Nyquist theorem and sample rates in practice
- [x] Decimation and downsampling
- [x] Digital filters: low-pass, band-pass, FIR vs IIR
- See: [research/sdr-fundamentals.md](sdr-fundamentals.md)

## FM Demodulation
- [x] Phase difference demodulation (the atan2 method)
- [x] De-emphasis filter (75 μs in North America)
- [x] Stereo FM decoding (pilot tone at 19 kHz)
- [x] Wideband FM vs narrowband FM
- See: [research/sdr-fundamentals.md](sdr-fundamentals.md)

## CHU Time Signal
- [x] CHU transmission format and protocol
- [x] BCD time code encoding
- [x] Tone patterns (1 second ticks, voice announcements)
- [x] Comparison with WWV/WWVB (US equivalents)
- [x] NRC documentation on CHU
- See: [research/chu-time-signal.md](chu-time-signal.md)

## RTL-SDR Hardware
- [x] RTL2832U chipset and R820T2 tuner (covered in sdr-fundamentals.md)
- [x] Frequency range and limitations (covered in sdr-fundamentals.md)
- [x] USB data transfer — bulk transfers, buffer sizes (covered in sdr-fundamentals.md)
- [ ] rtl_tcp protocol for network SDR access
- [ ] Direct sampling mode for HF (shortwave) reception

## Rust Crate Evaluation
- [ ] `rtl-sdr-rs` — API, maturity, limitations
- [ ] `rtlsdr_mt` — threading model, comparison with rtl-sdr-rs
- [ ] `demod_fm` — does it actually work well?
- [ ] `rust_radio` — what does it provide?
- [ ] `dasp` — useful for audio output stage?
- [ ] `soapysdr` — worth the abstraction overhead?
- [ ] `cpal` or `rodio` — for audio output
- [ ] `ratatui` — for the scanner TUI display

## Signal Identification
- [ ] sigidwiki.com — reference for identifying unknown signals
- [ ] Common signals in the Ottawa area
- [ ] What makes a good "tour" of the spectrum for the talk
