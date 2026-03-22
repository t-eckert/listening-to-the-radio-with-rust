# Listening to the Radio with Rust — Talk Outline

**Title:** Listening to the Radio with Rust
**Duration:** 35–40 minutes
**Venue:** Ottawa Systems, 24 March 2026
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

Hello, my name is Thomas Eckert. I am a software engineer at Redpanda. My background is in physics. I am always picking up new hobbies and interests.

I'm very excited to share with you today about a new hobby that I've picked up: software defined radio. I am not an expert, this is something I've been playing with for the past few months.

When I find something cool, my first instinct is to want to share it with others. This is a technology that is super accessible.

---

## 1. The Physics — EM Waves (5 min)

### The bobber metaphor

Imagine a bobber sitting in a pool of water. You push it up and down — it oscillates and creates waves that radiate outward across the surface.

This is what happens when charged particles accelerate. An electron moving up and down in a wire creates electromagnetic waves that radiate outward through space.

### The receiver

Now imagine a second bobber sitting in the same pool, some distance away. The waves reach it. It begins to oscillate too — driven by the energy carried in the waves.

This is the receiving antenna. Electrons in the metal are pushed up and down by the incoming EM wave. The antenna converts the wave back into electrical current.

### Grounding the audience

The audience should be able to picture this: electrons moving up and down in a transmitting antenna create waves; those waves push electrons up and down in a receiving antenna. Everything that follows in this talk is about measuring and interpreting that movement.

---

## 2. The RTL-SDR — Two Chips (3 min)

### What is SDR?

- Traditional radio: one hardware circuit, one purpose
- SDR: digitize a chunk of spectrum, do everything else in software
- Same dongle receives FM, AM, aviation, ADS-B — just change the frequency and the code

### The hardware path

```
Antenna → R828D tuner → RTL2832U → USB → your code
```

- **R828D tuner**: Takes the high-frequency RF signal and shifts it down to a lower frequency the ADC can handle. It's the "ear" — selects which frequency range to listen to.
- **RTL2832U**: An 8-bit analog-to-digital converter. Samples the signal and sends digital data over USB. Originally a DVB-T TV tuner chip — hackers figured out you could grab raw samples from the ADC. The RTL-SDR was born.

Keep this high-level. Two chips, one job each: tune and digitize.

---

## 3. IQ Samples — Points on the Complex Plane (5 min)

### What comes out of the dongle

The RTL2832U outputs pairs of bytes: I, Q, I, Q, ...

Each pair is a point on the complex plane: `I + jQ`.

### The rotation

These points trace rotation around the origin.

- **Speed of rotation** → proportional to the frequency of the wave
- **Distance from the origin** → proportional to the amplitude of the wave

This is the key insight. Frequency is rotation speed. Amplitude is distance from center. Everything we do with SDR is about measuring these two properties.

### The explicit omission

"We won't go into the deep mathematical detail of how the hardware produces these values — that involves mixing, downconversion, and the Hilbert transform. What matters for us is what the values *mean*."

### Demo: iq-print (2 min)

Show raw IQ values streaming in from the dongle. The audience sees the data they've been hearing about.

---

## 4. Demodulation — Many Signals, One Idea (3 min)

### The framing

All of the different signals sent over radio are interpreted through different methods of demodulation. But they all come back to measuring either the rotation speed or the distance from the origin — or both.

### The chart

Show the two tables: magnitude-based signals and phase-based signals.

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
| FSK (CHU, pagers) | Various | Digital bits |
| AIS (ships) | 162 MHz | Ship position and identity |
| POCSAG pagers | 148/152 MHz | Text messages |

---

## 5. Demo: FM Radio (3 min)

### The demo

Play a local FM station live through the laptop speakers.

- Show the code. The entire FM demodulation pipeline fits on one screen.
- The key line: `let audio = (sample * prev.conj()).arg();`
- "That's FM demodulation. Multiply by the conjugate of the previous sample, take the angle. The audio IS the rate of phase change — the rotation speed."

### Tie it back

This is a phase-based signal. We're measuring how fast the point rotates around the origin. The speed of rotation encodes the audio.

---

## 6. Demo: AM Radio (3 min)

### The demo

Tune to an AM station and play the audio.

- The key operation: `sqrt(I² + Q²)` — just take the magnitude.
- "AM is even simpler. The audio is the distance from the origin. The amplitude of the wave IS the signal."

### The contrast

"Both FM and AM produce audio. But FM uses the variation of speed in rotation around the origin to encode the audio. AM uses the distance from the origin. Same IQ data, different interpretation."

---

## 7. Antenna Length — Why It Matters (4 min)

**Staging note:** During this section, swap the long dipole antenna for the short 7cm ADS-B antenna. The physical act of changing antennas reinforces the point.

### The physics

- An antenna works best when its length is related to the wavelength of the signal.
- The sweet spot: **1/4 of the wavelength**. At this length, the antenna resonates — the electrons oscillate with maximum efficiency.

### Why not longer?

- If the antenna is too long relative to the wavelength, **eddy currents** form. Parts of the antenna work against each other — current flows in opposing directions, canceling out the signal.
- This is why you don't use the same antenna for FM (88 MHz, ~3.4m wavelength) and ADS-B (1090 MHz, ~27cm wavelength). The FM dipole is useless at 1090 MHz.

### The practical point

"I'm swapping antennas right now. This short antenna is about 7cm — roughly a quarter wavelength at 1090 MHz. It's optimized for ADS-B."

---

## 8. Demo: ADS-B — Aircraft Tracking (5 min)

### The demo

Run the ADS-B decoder with the short antenna. Show aircraft appearing on screen with callsign, altitude, position, and speed.

- ADS-B is a magnitude-based signal — on/off keying at 1090 MHz.
- Every aircraft with a transponder broadcasts its position, altitude, speed, and callsign. Unencrypted. Twice per second.
- Show the Ottawa area map overlay with aircraft positions.

### The impact

"Every plane in the sky above us is announcing itself right now. With a $30 dongle and a 7cm antenna, we can see them all."

---

## 9. CHU — Ottawa's Atomic Clock (5 min, if working)

**Note:** CHU decoder had issues earlier. Include if working, skip gracefully if not.

### The story

- CHU is a shortwave time signal station operated by NRC Canada.
- Broadcasting continuously since 1938. Transmitter site in Barrhaven, about 10 miles from here.
- Three frequencies, three cesium atomic clocks.
- Every second: a 1000 Hz tick. Every minute: a bilingual voice announcement. Every second from 31-39: a 300-baud FSK data burst encoding the exact time.

### The demo

Run the CHU decoder. Show ticks accumulating, then the time appearing.

"That time came from a cesium clock in Barrhaven, through the air, into a $30 dongle, through a few hundred lines of Rust. No internet. No GPS. No NTP. Just radio waves and math."

### If it doesn't work

"CHU operates on shortwave at 7.85 MHz — reception depends on atmospheric conditions and the antenna. I've decoded it successfully from home, but live RF is unpredictable. That's part of the fun."

---

## 10. Closing (2 min)

### What we covered

- EM waves: electrons oscillating in antennas, creating and receiving waves
- The RTL-SDR: two chips that digitize radio into IQ samples
- IQ samples: points on the complex plane where rotation speed is frequency and distance is amplitude
- Demodulation: FM (phase), AM (magnitude), ADS-B (magnitude), CHU (FSK/phase)
- Antenna design: why length matters and quarter-wavelength resonance

### The invitation

"Everything I showed you today costs about $30 in hardware and runs on any laptop. The RTL-SDR Blog V4 dongle, a few Rust crates, and curiosity. Go listen to what's in the air."

### Open discussion

- Pass around the hardware
- Questions, ideas, war stories
- "Has anyone here done amateur radio or SDR?"

---

## Production Notes

### Equipment to bring
- RTL-SDR Blog V4 dongle
- Long dipole antenna (FM/AM/CHU)
- Short 7cm antenna (ADS-B)
- Laptop (primary demo machine)
- USB-C adapter if needed for venue display

### Demo risk mitigation
- **Pre-record IQ samples** for all demos. The code reads from a file source identically to a live source — swap one line.
- **Pre-record terminal sessions** (asciinema) as last-resort backup.
- **Test at the venue** if possible — RF environment varies.
- **Build release binaries** ahead of time. Don't compile on stage.

### If time is tight
- Cut CHU (section 9). The talk stands without it.
- Shorten the antenna theory section — state the 1/4 wavelength rule without the eddy current detail.
- Core path: Physics (5) → SDR (3) → IQ (5) → Demod (3) → FM (3) → AM (3) → Antenna (2) → ADS-B (5) → Close (2) = 31 min

### If time is generous
- Let ADS-B run longer. Watching aircraft accumulate is satisfying.
- Deeper dive into the IQ → complex plane connection with visuals.
- Show the Rust crate ecosystem: rtl-sdr-rs, rustfft, cpal. You end up writing the interesting parts yourself — which is a feature, not a bug, for this audience.
