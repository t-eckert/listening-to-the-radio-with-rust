# Listening to the Radio with Rust — Talk Outline

**Title:** Listening to the Radio with Rust
**Duration:** 35–40 minutes
**Venue:** Bayview Yards, Ottawa Systems, 24 March 2026
**Goal:** Introduce technical folks to SDR in a way that enables them to go experiment themselves.

---

## Narrative Arc

The talk builds understanding layer by layer:

1. **Physics** — what is physically happening in an antenna
2. **Instrumentation** — how the RTL-SDR digitizes that physical phenomenon
3. **Interpretation** — how different demodulation methods extract meaning from the same data
4. **Application** — live demos of increasing complexity, culminating in ADS-B aircraft tracking

The audience should leave understanding *why* SDR works, not just *that* it works.

---

## Personal Intro (1 min)

> "I'm not an expert. This is something I've been playing with for a few months. When I find something cool, my first instinct is to share it."

- Software engineer at Redpanda, background in physics
- SDR is a new hobby — super accessible
- Set the tone: this is a hobby talk, not a lecture

**If you drift:** You're here to share something cool. That's it.

---

## 1. The Physics — EM Waves (5 min)

> "Imagine a bobber in a pool of still water."

### The transmitter

- Push the bobber up and down → it oscillates → waves radiate outward
- This is what charged particles do. An electron moving up and down in a wire creates EM waves.

### The receiver

- Second bobber in the same pool. The waves reach it. It starts oscillating too.
- That's the receiving antenna. Electrons pushed up and down by the incoming wave.

### The summary — run wave-demo here

> "Everything that follows is about measuring and interpreting that movement."

**If you drift:** Two bobbers. One makes waves, one receives them. Electrons up and down.

---

## 2. The RTL-SDR — Two Chips (3 min)

> "Traditional radio: one circuit, one purpose. SDR: digitize the spectrum, do everything else in software."

### What is SDR?

- Same dongle receives FM, AM, aviation, ADS-B — just change the frequency and the code

### The hardware path

```
Antenna → R828D tuner → RTL2832U → USB → your code
```

- **R828D**: selects which part of the spectrum to listen to
- **RTL2832U**: 8-bit ADC, sends digital data over USB

> "Originally a DVB-T TV tuner chip. Hackers figured out you could grab raw samples from the ADC. The RTL-SDR was born."

**If you drift:** Two chips. One tunes, one digitizes. That's the whole dongle.

---

## 3. IQ Samples — Points on the Complex Plane (5 min)
 
> "The dongle gives you pairs of bytes. Each pair is a point on the complex plane."

### The concept — this is the key slide

- These points trace rotation around the origin
- **Speed of rotation** → frequency
- **Distance from origin** → amplitude

> "Frequency is rotation speed. Amplitude is distance from center. Everything we do with SDR is about measuring these two properties."

### Demo: iq-demo

- Run the interactive visualization
- Arrow keys to change frequency (rotation speed) and amplitude (circle size)
- Let the audience see the connection

### The code

- Show the bytes_to_iq conversion — raw bytes in, complex numbers out

### The omission

> "We won't go into the deep math of how the hardware produces these values. What matters is what they mean."

### Demo: iq-print

- Show real IQ data streaming from the dongle
- "These are the numbers. Every demo that follows processes these."

**If you drift:** Points on a plane. Speed = frequency. Distance = amplitude. That's the whole game.

---

## 4. Demodulation — Many Signals, One Idea (3 min)

> "All demodulation comes back to measuring rotation speed or distance from the origin."

### The tables

**Magnitude-based (distance from origin):**

| Signal | Frequency | What you get |
|--------|-----------|-------------|
| AM broadcast | 530–1700 kHz | Audio |
| ADS-B | 1090 MHz | Aircraft position, altitude, speed |
| OOK | 315/433 MHz | Garage doors, weather stations |
| NOAA APT | 137 MHz | Satellite weather images |

**Phase-based (rotation speed):**

| Signal | Frequency | What you get |
|--------|-----------|-------------|
| FM broadcast | 88–108 MHz | Audio |
| FSK (pagers, telemetry) | Various | Digital bits |
| AIS (ships) | 162 MHz | Ship position and identity |
| POCSAG pagers | 148/152 MHz | Text messages |

> "Let's hear some signals."

**If you drift:** Two columns. Magnitude or phase. Everything fits in one of them.

---

## 5. Demo: FM Radio (3 min)

> "Multiply by the conjugate of the previous sample. Take the angle. That's FM demodulation."

### The demo

- Play a local FM station. Try 106.1 (CHEZ, classic rock) or 100.3 (Majic).
- Let it play for a few seconds. Let the audience hear it.

### The code

- The key line: `(sample * prev.conj()).arg()`
- The audio IS the rate of phase change — the rotation speed

> "We're measuring how fast the point rotates around the origin. That speed is the audio."

**If you drift:** One line of math. Phase change is audio. Play the music, show the code.

---

## 6. Demo: AM Radio (3 min)

> "AM is even simpler. Just take the magnitude. sqrt(I² + Q²). That's it."

### The demo

- Tune to 118.8 MHz — YOW tower. Vertical antenna.
- You may hear ATC in English or French — Ottawa is bilingual.
- If tower is quiet, narrate: "We're listening to the Ottawa airport tower frequency. When a pilot or controller transmits, we'll hear them."

### The contrast

> "Both FM and AM produce audio. FM uses the speed of rotation. AM uses the distance from the origin. Same IQ data, different interpretation."

**If you drift:** Magnitude = AM. One function call. Then contrast with FM.

---

## 7. Antenna Length — Why It Matters (4 min)

> "An antenna resonates at a quarter of the wavelength."

**Swap the antenna during this section.** The physical act reinforces the point.

### The physics

- Antenna works best at 1/4 of the wavelength
- Too long → destructive interference → parts of the antenna cancel each other out

### The examples

- FM (88 MHz) → wavelength ~3.4 m → dipole ~85 cm per side
- ADS-B (1090 MHz) → wavelength ~27 cm → antenna ~7 cm

> "I just swapped antennas. This short one is about 7 cm — a quarter wavelength at 1090 MHz."

**If you drift:** Quarter wavelength. Too long = destructive interference. Show the short antenna.

---

## 8. Demo: ADS-B — Aircraft Tracking (5 min)

> "Every plane in the sky is announcing itself right now."

### The demo

- Run adsb-decoder, then flight-tracker
- Aircraft appear on the Ottawa map with callsign, altitude, speed
- Let it accumulate. Each new dot is satisfying.

### The explanation

- ADS-B is magnitude-based — same `s.norm()` as AM
- On/off keying at 1090 MHz, unencrypted, twice per second
- 120 μs burst → pulse positions → lat, lon, altitude

> "With a $30 dongle and a 7 cm antenna, we can see them all."

**If you drift:** Same math as AM, different protocol on top. Show the map. Let it fill in.

---

## 9. Closing (2 min)

> "Everything I showed you today costs about $30 and runs on any laptop."

### Quick recap (don't belabor — the slides list it)

- EM waves, two chips, IQ samples, demodulation, antennas
- Four live demos from one USB dongle

### The invitation

> "The airwaves are public. The signals are free. The tools are open source. Go listen."

### Open discussion

- Pass around the hardware
- "Has anyone here done amateur radio or SDR?"
- Questions, ideas, war stories

**If you drift:** $30 dongle, a few crates, curiosity. Go listen.

---

## Panic Card

If you completely lose your place, say one of these and you'll find the thread again:

- **"So, what does this mean practically?"** → transition to the next demo
- **"Let me show you."** → switch to a demo, any demo
- **"The key idea is..."** → rotation speed is frequency, distance is amplitude
- **"Let's go back to the IQ plane."** → you can always re-anchor on the core concept

You know this material. You built every demo. The audience is on your side.

---

## Production Notes

### Equipment to bring
- RTL-SDR Blog V4 dongle
- Long dipole antenna (FM/AM — vertical for AM)
- Short 7cm antenna (ADS-B)
- Laptop (primary demo machine)
- USB-C adapter if needed for venue display

### Pre-talk setup
- `task build` — build release binaries
- `task alias` — create short names in demos/bin/
- Start `rtl_tcp` in a background terminal
- Test FM receiver to verify audio output works at the venue
- Have presenterm slides open: `task present`

### Demo commands (quick reference)
- `task wave` — EM wave animation
- `task iq` — interactive IQ visualization (arrow keys)
- `task iq-print` — raw IQ from dongle
- `task fm FREQ=106.1` — FM radio (CHEZ classic rock)
- `task am FREQ=118.8` — AM aviation (YOW tower)
- `task adsb` — ADS-B decoder (then `task tracker` in second terminal)

### Demo risk mitigation
- **Pre-record IQ samples** for all demos. The code reads from a file source identically to a live source — swap one line.
- **Pre-record terminal sessions** (asciinema) as last-resort backup.
- **Test at the venue** if possible — RF environment varies.
- **Build release binaries** ahead of time. Don't compile on stage.

### If time is tight
- Shorten the antenna theory section — state the 1/4 wavelength rule without the destructive interference detail.
- Core path: Physics (5) → SDR (3) → IQ (5) → Demod (3) → FM (3) → AM (3) → Antenna (2) → ADS-B (5) → Close (2) = 31 min

### If time is generous
- Let ADS-B run longer. Watching aircraft accumulate is satisfying.
- Deeper dive into the IQ → complex plane connection with iq-demo.
- Show the Rust crate ecosystem: rtl-sdr-rs, cpal, ratatui. You end up writing the interesting parts yourself — which is a feature, not a bug, for this audience.

### If a demo fails
- Don't apologize at length. Say: "Live RF — that's part of the fun." Move on.
- You can always fall back to wave-demo and iq-demo — they need no hardware.
- The talk works even if every live demo fails. The physics, the concepts, and the code slides carry the story.
