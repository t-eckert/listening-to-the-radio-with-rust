# Listening to the Radio with Rust — Talk Outline

**Title:** Listening to the Radio with Rust
**Duration:** 35 minutes + open discussion
**Throughline:** "What's hiding in plain sight?"

---

## Narrative Arc

The talk moves from the familiar to the invisible to the revelatory:

1. Start with something everyone knows (FM radio) → make it tangible with code
2. Pull back and show the spectrum is teeming with signals → what else is out there?
3. Land on CHU — a 1938 atomic clock broadcasting the time from 10 miles away, decoded live

The audience should leave thinking: "the air around me is full of data I've never noticed, and I could build something to read it this weekend."

---

## Personal Intro

Hello, my name is Thomas Eckert. I am a software engineer at Redpanda. My background is in physics. I am always picking up new hobbies and interests. 

I'm very excited to share with you today about a new hobby that I've picked up: software defined radio. I am not an expert, this is something I've been playing with for the past few months.

When I find something cool, my first instinct is to want to share it with others. This is a technology that is super accessible.

---

## The Hook — FM from 50 Lines of Rust (3 min)

Audio starts playing — a local FM station coming through the laptop speakers.

- Pause. Let it play for a few seconds.
- "This is live. That's an antenna plugged into a $30 USB dongle, and about 50 lines of
  Rust reading IQ samples and doing one line of math."
- Show the dongle. Show the code. The entire FM demodulation pipeline fits on one screen.
- The key line: `let audio = (sample * prev.conj()).arg();`
- "That's it. That's FM demodulation. Multiply by the conjugate of the previous sample,
  take the angle. The audio IS the rate of phase change."

**Why this works as an opener:** Immediate proof that this isn't theoretical. The audience
hears real radio. The code is short enough to be believable. And you've planted the hook:
if FM radio is this simple, what else can you do?

---

## 2. What's in the Air Right Now (5 min)

**Transition:** "FM is the obvious one. But the spectrum doesn't stop at 108 MHz."

Show a spectrum view (pre-captured or live from the scanner) of what's in the airwaves
right now. Walk through it:

- **88–108 MHz** — FM broadcast. The thing we just heard.
- **118–137 MHz** — Aviation AM. Tower, ground, approach. Planes talking to
  controllers nearby.
- **144–148 MHz** — 2-meter ham band. Local repeaters, maybe someone chatting right now.
- **1090 MHz** — ADS-B. Every aircraft in the sky broadcasting its position, altitude,
  and speed. Unencrypted. Your phone can't hear it, but this dongle can.
- **3.33 / 7.85 / 14.67 MHz** — Shortwave. Tease this: "We'll come back to these."

**The point:** All of this is in the air in this room right now. The only difference
between hearing it and not is whether you have the right tool. An SDR is that tool.

### What is SDR? (1 min, folded in)

- Traditional radio: one hardware circuit, one purpose
- SDR: digitize a wide chunk of spectrum, do everything else in software
- Same dongle receives FM, aviation, ADS-B, shortwave — just change the frequency
  and the processing code
- Originally a $20 DVB-T TV tuner. Hackers figured out you could grab raw samples
  from the ADC. The RTL-SDR was born.

---

## 3. How the Data Flows — Antenna to Iterator (5 min)

The systems-depth section. The full path from antenna to Rust code.

### The hardware path (2 min)

```
Antenna → R828D tuner (amplify, downconvert) → RTL2832U (8-bit ADC, USB) → your code
```

- R828D tuner: LNA + mixer + PLL. Shifts RF down to baseband. Controlled via I2C.
- RTL2832U: 8-bit ADC sampling at 28.8 MHz internally, decimates to your requested
  rate (typically 2.4 MS/s). Outputs interleaved unsigned bytes: I, Q, I, Q, ...
- USB bulk transfers: 15 in-flight buffers of 256 KB each. Your callback must keep up
  or you drop samples.
- **8 bits is the constraint.** ~40 dB dynamic range. This is why gain control matters
  and why you can't hear a weak signal next to a strong one.

### The data (2 min)

- Each byte pair is one complex sample: `I + jQ`
- Raw: unsigned u8 [0, 255]. Convert: `(raw as f32 - 127.5) / 127.5` → [-1.0, 1.0]
- At 2.4 MS/s: 4.8 MB/s of IQ data flowing through your Rust code
- Show the conversion code. Highlight `chunks_exact(2)` + `map` — this is an iterator
  pipeline, zero allocation, processes millions of samples per second.

### The connection model (1 min)

Show two paths — both produce the same `&[u8]` stream:

```
Path A: USB direct (rtl-sdr-rs)     Path B: Network (rtl_tcp)
  RTL-SDR → USB → read_async()       RTL-SDR → USB → Pi → TCP → read()
```

- rtl_tcp is a ~80-line client. Dead simple protocol: 12-byte header, 5-byte commands,
  then a firehose of IQ bytes.

---

## 4. Scanning the Spectrum (4 min)

**Transition:** "So we heard FM. Let's see what else is out there."

### Live demo: freq-scanner

Run the scanner TUI. It sweeps a frequency range and shows signal power at each step.

- Sweep the FM band (88–108 MHz). Bars light up where stations are.
- Sweep the aviation band (118–137 MHz). Maybe catch a transmission.
- The scanner uses FFT (rustfft) to compute power spectral density per step.
  1024-point FFT, Hamming window, magnitude squared.

### Show the code briefly

- The sweep loop: retune → discard stale buffer → collect → FFT → measure power
- Highlight: this is retuning the hardware hundreds of times per second. The PLL
  settling time (~5 ms) is the bottleneck, not the Rust code.

**Purpose of this demo:** It's a bridge. The scanner makes the invisible visible. It
shows there are signals everywhere. And it naturally raises the question: what ARE those
signals? Which leads to...

---

## 5. The Reveal — CHU, Ottawa's Atomic Clock (8 min)

**Transition:** "Remember those shortwave frequencies I skipped? 3.33, 7.85, 14.67 MHz.
Let's go back to them."

### The story of CHU (2 min)

- CHU is a shortwave time signal station operated by NRC Canada.
- Broadcasting continuously since 1938. Three frequencies, three antennas, three
  cesium atomic clocks.
- Transmitter site: Barrhaven. About 10 miles from here.
- Every second: a 1000 Hz tick. Every minute: a bilingual voice announcement
  ("At the tone, Eastern Daylight Time will be..."). Every second from 31-39:
  a 300-baud FSK data burst encoding the exact time in BCD.
- The carrier frequency is accurate to 5 parts per trillion. Derived from cesium.
- This signal is in the air in this room right now.

### How the decoder works (3 min)

Walk through the decoding chain — this is where the systems depth shines:

```
7.850 MHz RF → AM demod (envelope detection) → audio stream
  → bandpass filter (2025–2225 Hz) → FSK discriminator
  → UART framing (8N2 @ 300 baud) → nibble swap → BCD parse → UTC time
```

- AM demod is simpler than FM: just `sqrt(I² + Q²)`. Take the magnitude.
- FSK discrimination: Goertzel filter at 2025 Hz (space/0) and 2225 Hz (mark/1).
  Compare magnitudes. That's Bell 103 modem decoding — a 1960s protocol.
- UART framing: detect start bit, sample 8 data bits, check 2 stop bits.
  You're building a software modem.
- Each burst: 10 bytes, transmitted twice for redundancy. Nibble-swap, parse BCD.
  One burst gives you year, day-of-year, hour, minute, second.
- 8 bursts per minute. You get a valid time in under 10 seconds.

Show key code: the Goertzel filter (10 lines), the state machine enum, the BCD parser.

### Live demo (3 min)

Run the CHU decoder. The terminal shows:

```
CHU Time Signal Decoder — 7.850 MHz
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Listening...
  ● Tick detected (1000 Hz, 300ms)
  ● Tick detected
  ...
  ◆ Data burst received (second 32)
    Format A: Day 083, 01:47:32 UTC

  UTC:   2026-03-24 01:47:32
  Local: 2026-03-23 21:47:32 EDT

  Source: CHU Ottawa (NRC cesium clock)
  Accuracy: ±100 μs (broadcast) + propagation
```

Let it run. Let the ticks accumulate. Let the time appear.

**The landing:** "That time came from a cesium clock in Barrhaven, through the air,
into a $30 dongle, through a few hundred lines of Rust. No internet. No GPS. No NTP.
Just radio waves and math."

---

## 6. Closing + Discussion (open-ended)

### Quick recap (1 min)

- We heard FM radio from one line of phase math
- We scanned the spectrum and saw the invisible
- We decoded atomic time from a signal that's been broadcasting since 1938
- All of it: one USB dongle, pure Rust, no frameworks, no C dependencies

### "What else could you build?" (transition to discussion)

Seed ideas:
- ADS-B aircraft tracker (1090 MHz, there are Rust decoders)
- NOAA weather satellite image decoder (137 MHz APT)
- Pager decoding (POCSAG/FLEX)
- LoRa packet sniffer
- Your own ham radio transceiver (with a HackRF or similar TX-capable SDR)

### Open discussion

- Pass around the hardware
- Questions, war stories, ideas
- "Has anyone here done amateur radio or SDR?"

---

## Production Notes

### Equipment to bring
- RTL-SDR Blog V4 dongle + dipole antenna
- Laptop (primary demo machine)
- Optional: Raspberry Pi 4 B running rtl_tcp (backup / network demo)
- USB-C adapter if needed for venue display

### Demo risk mitigation
- **Pre-record IQ samples** for all three demos. The code reads from a file source
  identically to a live source — swap one line.
- **Pre-record terminal sessions** (asciinema) as last-resort backup.
- **Test at the venue** if possible — RF environment varies. Some venues have heavy
  RF shielding.
- **Build release binaries** ahead of time. Don't compile on stage.

### If time is tight
- Cut the scanner demo (section 4). Go directly from "what's in the air" to CHU.
  Show a pre-captured spectrum screenshot instead of a live scan.
- The talk works as: Hook (3 min) → Context (5 min) → Systems depth (5 min) →
  CHU (8 min) → Close (2 min) = 23 min without the scanner.

### If time is generous
- Let the CHU decoder run longer. Each tick is satisfying.
- Deeper dive into the rtl_tcp protocol (show the 5-byte command format, the 12-byte
  header parsing).
- Show the Rust crate ecosystem honestly: rtl-sdr-rs is solid, DSP crates are thin,
  you end up writing the interesting parts yourself (which is a feature, not a bug,
  for this audience).
