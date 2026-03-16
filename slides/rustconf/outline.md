# RustConf 2026 — Talk Outline

**Title:** Listening to the Radio with Rust
**Duration:** 40 minutes (35 content + 5 optional Q&A)
**Audience:** Beginner/Intermediate Rust developers, no radio/DSP background assumed

---

## Act 1 — What's on the Air (~10 min)

### 1.1 Opening hook (2 min)
- Live demo or recording: tune to a local FM station, audio plays from the laptop
- "This is coming from a $30 USB dongle and about 50 lines of Rust."
- Pause. Let it land.

### 1.2 The invisible world (4 min)
- The radio spectrum is enormous and full of stuff
- Visual: spectrum diagram from ~30 kHz to 1.5 GHz
- Walk through what lives where:
  - **FM broadcast** (88–108 MHz) — music, news, the familiar stuff
  - **Aviation** (118–137 MHz AM) — tower, ground control, approach
  - **Ham/amateur radio** (144–148 MHz, 420–450 MHz) — repeaters, local operators
  - **Weather** — NOAA satellite imagery on 137 MHz
  - **ADS-B** (1090 MHz) — aircraft positions, altitude, speed
  - **Time signals** — CHU Ottawa on 3.33/7.85/14.67 MHz shortwave
- Key point: all of this is in the air around you right now

### 1.3 What is SDR? (2 min)
- Traditional radio: hardware tuner + analog demodulator, one purpose
- SDR: digitize a chunk of spectrum, do everything in software
- This is why a single $30 dongle can receive all of the above
- The RTL-SDR: originally a DVB-T TV tuner, repurposed by hackers

### 1.4 The hardware (2 min)
- Show the RTL-SDR dongle (photo or hold it up)
- Antenna options: telescopic dipole, purpose-built for different bands
- Connection: USB → computer → your code
- That's it. That's the whole hardware stack.

---

## Act 2 — SDR from First Principles (~10 min)

### 2.1 From antenna to numbers (3 min)
- Antenna receives electromagnetic waves → voltage signal
- ADC (analog-to-digital converter) in the dongle samples the voltage
- Output: a stream of IQ (In-phase / Quadrature) samples
- Diagram: sine wave → sampled points → complex numbers
- Why complex? Because you need both amplitude AND phase to reconstruct the signal

### 2.2 What are IQ samples, really? (3 min)
- Each sample is a pair: (I, Q) — two numbers
- Think of it as a point on a circle (or a complex number)
- The I component = cosine, Q component = sine
- Together they capture the full instantaneous state of the signal
- Data rate: at 2.4 MHz sample rate, that's 2.4 million complex samples per second
- This is why performance matters — and why Rust is a good fit

### 2.3 From samples to sound (4 min)
- FM demodulation in one slide:
  - FM encodes audio as changes in frequency
  - Measure the rate of phase change between consecutive samples
  - That rate IS the audio signal
  - `atan2(I[n]*Q[n-1] - Q[n]*I[n-1], I[n]*I[n-1] + Q[n]*Q[n-1])`
- Diagram: IQ samples → phase differences → audio waveform → speaker
- This is the entire FM demodulation algorithm. Everything else is filtering and cleanup.

---

## Act 3 — Building Three Programs in Rust (~15 min)

### 3.1 Demo 1: FM Receiver (5 min)
- **Goal:** Tune to a broadcast FM station and hear it
- Walk through the code:
  - Connect to RTL-SDR device (or rtl_tcp server)
  - Set center frequency to a known FM station
  - Read IQ samples in a loop
  - Apply FM demodulation (phase difference)
  - Downsample to audio rate (48 kHz)
  - Write to audio output
- Run it. Audio plays.
- Highlight Rust strengths:
  - Iterator chains for the DSP pipeline
  - Type safety: sample rate as a type parameter or newtype
  - No allocations in the hot path

### 3.2 Demo 2: CHU Time Signal Decoder (5 min)
- **Goal:** Tune to CHU Ottawa (3.33 MHz / 7.85 MHz) and decode the time
- Context: CHU is a real time station operated by NRC, 10 miles from downtown Ottawa
- Walk through the code:
  - Tune to CHU frequency
  - Narrowband filter to isolate the signal
  - Detect the time code tone pattern (1 second ticks, voice announcement)
  - Parse the BCD time code from the data burst
  - Print the decoded UTC time
- Run it. Time appears.
- Highlight Rust strengths:
  - Pattern matching for state machine (detecting tone sequences)
  - Enums for signal states
  - Composable filter chains

### 3.3 Demo 3: Frequency Scanner (5 min)
- **Goal:** Sweep a frequency range and show what's active
- Walk through the code:
  - Define a frequency range (e.g., 88–108 MHz for FM band)
  - Step through in increments, tuning the SDR to each frequency
  - Collect samples at each frequency
  - Compute signal power (magnitude squared, averaged)
  - Display results: frequency vs. power, highlight active channels
- Run it. Terminal shows a spectrum display of active stations.
- Highlight Rust strengths:
  - Performance: scanning many frequencies quickly
  - Concurrency: could parallelize with async or threads
  - Terminal UI with ratatui or similar for live display

---

## Wrap-up (~5 min)

### What we covered
- The radio spectrum is vast and accessible
- SDR replaces hardware with software — one dongle, infinite receivers
- Rust is well-suited: performance, safety, expressive DSP pipelines

### Getting started
- Hardware: RTL-SDR Blog V4 (~$30), dipole antenna kit (~$10)
- Software: `rtl-sdr-rs`, `demod_fm`, `dasp`
- First project: FM receiver (the "hello world" of SDR)
- Resources: rtl-sdr.com, sigidwiki.com (signal identification), /r/RTLSDR

### Closing
- "The airwaves are public. The signals are free. The tools are open source. Go listen."

---

## Backup Plans

- **No live radio at venue:** Use pre-recorded IQ sample files. Code is identical — swap `SdrSource` for `FileSource`.
- **Hardware failure:** Pre-recorded terminal sessions (asciinema) of each demo running successfully.
- **Time running short:** Cut demo 3 (scanner) and show results as screenshots. Acts 1 and 2 + demos 1 and 2 work as a complete talk on their own.
