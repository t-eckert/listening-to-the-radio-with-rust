# CHU Time Signal Station -- Technical Reference

Comprehensive technical documentation on Canada's shortwave time signal station CHU,
operated by the National Research Council of Canada (NRC). This document is intended to
inform the design of a software-defined radio decoder written in Rust.

---

## Station Overview

**Operator:** Institute for National Measurement Standards, National Research Council of Canada (NRC)
**Call sign:** CHU
**Location:** Near Barrhaven, Ontario -- 15 km (10 mi) southwest of Ottawa's central business district
**Coordinates:** 45 17' 47" N, 75 45' 22" W
**First transmission:** 1938
**Broadcast:** Continuous, 24/7

CHU is one of only a handful of shortwave time signal stations in the world. From Ottawa, it is
essentially a local signal -- the transmitter is about 10 miles from downtown.

---

## Frequencies and Transmitter Power

| Frequency   | Power | Antenna          | Notes                                        |
|-------------|-------|------------------|----------------------------------------------|
| 3.330 MHz   | 3 kW  | Vertical, dedicated | Original frequency since 1938              |
| 7.850 MHz   | 5 kW  | Vertical, dedicated | Changed from 7.335 MHz on 2009-01-01       |
| 14.670 MHz  | 3 kW  | Vertical, dedicated | Original frequency since 1938              |

Each frequency has its own dedicated vertical antenna. The frequencies were deliberately
chosen to avoid interference with WWV (which uses 2.5, 5, 10, 15, 20, and 25 MHz).

The 7.335 MHz frequency was changed to 7.850 MHz in January 2009 after the ITU reallocated
the 7300-7350 kHz band from fixed service to general broadcast use in 2007, causing
interference from international broadcasters.

### Frequency Accuracy

Carrier frequency offsets from UTC(NRC) are less than 5 x 10^-12. The frequencies are derived
from one of a trio of cesium beam atomic clocks located at the transmitter site. Three clocks
are employed to permit majority logic checking (voting).

### Reception with RTL-SDR

All three CHU frequencies fall in the HF (shortwave) band, below the normal tuning range of
RTL-SDR dongles (which typically start around 24 MHz with the R820T2 tuner). To receive CHU:

- **Direct sampling mode**: Bypass the R820T2 tuner and feed the antenna signal directly to
  the RTL2832U ADC. This gives access to 0-14.4 MHz but with reduced sensitivity.
- **Upconverter**: A device like the Ham It Up shifts HF signals up by ~125 MHz into the
  tuner's native range. Better sensitivity than direct sampling.
- **RTL-SDR Blog V3/V4**: Has built-in direct sampling mode activated via software, no
  hardware modification needed.

The 7.850 MHz signal (5 kW) is the strongest and most reliable for software reception.

---

## Modulation

**Emission type:** H3E -- Upper Single Sideband (USB) with carrier reinserted

This is "AM-compatible" modulation: the upper sideband carries the audio content and the
carrier is reinserted so that a standard AM receiver can demodulate it (envelope detection
works). The lower sideband is suppressed.

### Implications for a Decoder

- **AM demodulation works**: Simple envelope detection (magnitude of the analytic signal)
  will recover the audio, including tones and voice announcements.
- **USB demodulation also works**: Mixing down and filtering the upper sideband gives cleaner
  audio with less noise.
- **For the digital time code**: The BCD data is encoded as audio-frequency FSK tones
  (2025/2225 Hz) within the audio passband, so once you demodulate the AM/USB carrier, the
  FSK tones appear in the audio stream and can be decoded with standard FSK techniques.

The demodulation chain is therefore:

```
RF signal (e.g., 7.850 MHz)
  -> Tune SDR center frequency
  -> IQ samples from RTL-SDR
  -> AM or USB demodulation (extract audio-band signal)
  -> Audio stream contains: 1000 Hz tones, FSK data bursts, voice
  -> Bandpass filter around 2025-2225 Hz for FSK extraction
  -> FSK demodulation (Bell 103 decoder)
  -> UART framing (start bit, 8 data bits, 2 stop bits)
  -> BCD time code parsing
```

---

## Minute Structure -- Second by Second

Each minute of CHU transmission follows a precise pattern. Understanding this is critical for
building a decoder state machine.

### Second 0: Minute Marker

- **Normal minutes (not top of hour):** 500 ms pulse of 1000 Hz tone
- **Top of hour (second 0 of minute 0):** 1000 ms (full second) pulse of 1000 Hz tone,
  followed by 9 seconds of silence (seconds 1-9)

### Seconds 1-16: DUT1-Encoded Ticks

- **Normal ticks:** 300 ms pulse of 1000 Hz tone at the start of each second
- **DUT1 "split" ticks:** The 300 ms tone is split into two shorter tones (a double beep),
  used to encode the difference between UT1 and UTC

DUT1 encoding:
- If DUT1 is positive (+0.1 to +0.8 s): seconds 1 through N are split (where N = DUT1 * 10)
- If DUT1 is negative (-0.1 to -0.8 s): seconds 9 through (8+N) are split
- Count the number of split tones to determine |DUT1| in tenths of a second
- Exception: seconds 1-9 are silent at the top of each hour

### Seconds 17-28: Normal Ticks

- 300 ms pulse of 1000 Hz tone at the start of each second
- No special encoding

### Second 29: Silence

- **No tone is transmitted.** The pulse at second 29 is always omitted.
- This silence distinguishes CHU from other time signal stations.

### Second 30: Normal Tick

- 300 ms pulse of 1000 Hz tone

### Seconds 31-39: Digital Time Code (BCD Data Bursts)

- **Tick:** Reduced to a 10 ms pulse of 1000 Hz at the start of each second
- **Data burst:** Immediately after the 10 ms tick, a Bell 103 FSK data burst is transmitted
  containing the BCD-encoded time code
- Second 31 transmits **Format B** (year, DUT1, leap second, DST info)
- Seconds 32-39 transmit **Format A** (day of year, hour, minute, second)

### Seconds 40-49: Normal Ticks

- 300 ms pulse of 1000 Hz tone at the start of each second

### Second 50-59: Voice Announcements

- **Tick:** Reduced to 10 ms pulse of 1000 Hz at the start of each second
- **Voice:** Bilingual station identification and time announcement
- Announces the upcoming minute (the minute that will begin at the next second 0)

Voice alternation pattern:
- **Odd minutes:** French first, then English
- **Even minutes:** English first, then French

The French voice is that of Simon Durivage (Radio-Canada news anchor).
The English voice is that of Harry Mannis (former CBC Radio announcer).
Digital voice recordings have been in service since 1990.

### Summary Table

| Seconds | Content                        | Tone Duration | Special              |
|---------|--------------------------------|---------------|----------------------|
| 0       | Minute marker                  | 500 ms        | 1000 ms at top of hr |
| 1-8     | Ticks (may be split for DUT1+) | 300 ms        | Split = DUT1 marker  |
| 9-16    | Ticks (may be split for DUT1-) | 300 ms        | Split = DUT1 marker  |
| 17-28   | Normal ticks                   | 300 ms        |                      |
| 29      | **Silence**                    | --            | Always omitted       |
| 30      | Normal tick                    | 300 ms        |                      |
| 31      | BCD data burst (Format B)      | 10 ms tick    | Year, DUT1, DST      |
| 32-39   | BCD data burst (Format A)      | 10 ms tick    | Day, HH:MM:SS        |
| 40-49   | Normal ticks                   | 300 ms        |                      |
| 50-59   | Voice announcement             | 10 ms tick    | Bilingual ID + time  |

---

## Digital Time Code -- BCD Encoding

This is the core of what a software decoder needs to parse.

### Transmission Parameters

| Parameter        | Value                                                    |
|------------------|----------------------------------------------------------|
| Modulation       | FSK (Frequency Shift Keying), Bell 103 standard          |
| Mark frequency   | 2225 Hz (represents binary 1)                            |
| Space frequency  | 2025 Hz (represents binary 0)                            |
| Baud rate        | 300 bps                                                  |
| Framing          | 8N2 (1 start bit, 8 data bits, 2 stop bits = 11 bits)   |
| Characters/burst | 10                                                       |
| Total bits/burst | 110 (10 characters x 11 bits)                            |
| Burst duration   | 366.67 ms (110 bits / 300 bps)                           |

### Timing Within Each Second (31-39)

```
0 ms        10 ms       133.3 ms                    500 ms     1000 ms
|-- tick --||-- mark --||----- 110 data bits -----||-- mark --||
  1000 Hz    2225 Hz     FSK data @ 300 baud        2225 Hz    silence
  10 ms      123.3 ms    366.7 ms                   ~10 ms
```

Precise breakdown:
1. **0-10 ms:** 1000 Hz tick (the second marker)
2. **10-133.3 ms:** Continuous 2225 Hz mark tone (allows modem/decoder to synchronize)
3. **133.3-500 ms:** 110 data bits at 300 baud (the actual time code)
4. **~500 ms:** Brief mark tone (~10 ms) to cleanly terminate the last stop bits
5. **500-1000 ms:** Silence until the next second

The data burst ends at precisely 500 ms past the second. This is a hard timing constraint
useful for synchronization.

### Packet Structure

Each burst contains 10 bytes (characters). The 10 bytes are organized as two identical
5-byte blocks for redundancy:

```
Byte:  [B0] [B1] [B2] [B3] [B4] [B5] [B6] [B7] [B8] [B9]
        |---- data block ----|  |--- redundancy block ---|
```

- **Format A** (seconds 32-39): Redundancy bytes are **identical** to the data bytes
- **Format B** (second 31): Redundancy bytes are the **one's complement** (bitwise NOT) of
  the data bytes

### Nibble Swap

**Important for implementors:** Each byte as received from the modem has its nibbles swapped
relative to the logical BCD digits. After receiving a byte, swap the high and low nibbles
before interpreting the BCD values.

```rust
fn nibble_swap(byte: u8) -> u8 {
    (byte >> 4) | (byte << 4)
}
```

### Format A -- Time of Day (Seconds 32-39)

After nibble swap, the 5 data bytes encode 10 BCD digits:

```
Byte 0: [6] [d1]    -- framing code (always 6) + hundreds digit of day-of-year
Byte 1: [d2] [d3]   -- tens and units digits of day-of-year
Byte 2: [h1] [h2]   -- tens and units digits of hour (UTC)
Byte 3: [m1] [m2]   -- tens and units digits of minute (UTC)
Byte 4: [s1] [s2]   -- tens and units digits of second (UTC)
```

Where:
- `6` is a constant framing code used for validation
- `ddd` = day of year (001-366)
- `hh` = hour (00-23) in UTC
- `mm` = minute (00-59)
- `ss` = second (32-39, matching the actual second of transmission)

**Example:** Day 045, 14:30:35 UTC would encode as:

```
After nibble swap: 60 45 14 30 35
As raw BCD digits: 6 0 4 5 1 4 3 0 3 5
```

Format A is transmitted 8 times per minute (seconds 32-39), with the `ss` field incrementing
each second. This massive redundancy allows for reliable decoding even with poor reception.

### Format B -- Year and Control Data (Second 31)

After nibble swap, the 5 data bytes encode:

```
Byte 0: [x] [z]     -- control flags + |DUT1| in tenths of second
Byte 1: [y1] [y2]   -- thousands and hundreds digits of year
Byte 2: [y3] [y4]   -- tens and units digits of year
Byte 3: [t1] [t2]   -- TAI-UTC offset in seconds (BCD)
Byte 4: [a1] [a2]   -- DST code for Canadian time zones
```

**The `x` nibble (control flags):**

```
Bit 3 (MSB): Even parity bit for this nibble
Bit 2:       Leap second warning -- one second will be SUBTRACTED
Bit 1:       Leap second warning -- one second will be ADDED
Bit 0 (LSB): Sign of DUT1 (0 = positive, 1 = negative)
```

**UT1 Calculation:**
```
If x bit 0 = 0:  UT1 = UTC + (z / 10) seconds
If x bit 0 = 1:  UT1 = UTC - (z / 10) seconds
```

**Other fields:**
- `yyyy` = Gregorian year (e.g., 2026)
- `tt` = TAI - UTC difference in seconds (BCD, e.g., 37 as of 2017)
- `aa` = Daylight saving time code for Canada (encodes DST status across all provinces)

**Example:** Year 2026, DUT1 = -0.2s, TAI-UTC = 37s, DST code 0x00:

```
x = 0001 (parity=0, no leap warning, DUT1 negative)
z = 2
After nibble swap: 12 20 26 37 00
```

### Validation Strategy for a Decoder

1. **Format A validation:**
   - First nibble must be `6` (framing code)
   - Compare 5 data bytes with 5 redundancy bytes (must be identical)
   - Second value `ss` must be in range 32-39
   - Day, hour, minute must be in valid ranges
   - Successive bursts (seconds 32-39) should show incrementing `ss` values

2. **Format B validation:**
   - Compare 5 data bytes with one's complement of 5 redundancy bytes
   - Check parity bit in `x` nibble
   - Year should be reasonable (e.g., 2020-2040)
   - TAI-UTC should be reasonable (currently 37 seconds)

3. **Cross-validation:**
   - Time from Format A should be consistent across all 8 bursts
   - Year from Format B should match expectations
   - DUT1 from Format B `x`/`z` nibbles should match DUT1 split-tone encoding in seconds 1-16

---

## Decoder Architecture

Based on the NTP reference implementation (driver7) and the signal structure, a CHU decoder
in Rust should have these major components:

### 1. AM/USB Demodulator

Extract the audio-band signal from the IQ samples:

```
IQ samples -> magnitude (AM) or frequency shift (USB) -> audio stream
```

For AM envelope detection: `audio[n] = sqrt(I[n]^2 + Q[n]^2)`

### 2. FSK Demodulator (Bell 103 Decoder)

Discriminate between 2025 Hz (space/0) and 2225 Hz (mark/1):

- **Bandpass filter**: Center on 2125 Hz, bandwidth ~400 Hz to isolate the FSK tones
- **Frequency discriminator**: Goertzel algorithm, or zero-crossing counter, or
  correlate against reference 2025 Hz and 2225 Hz tones
- **Decision threshold**: Compare energy at mark vs space frequency

The NTP reference implementation uses:
- 500 Hz bandpass filter centered on 2125 Hz
- Limiter/discriminator
- Raised-cosine lowpass filter optimized for 300 baud

A simpler approach for a demo:
- Sliding DFT (Goertzel) at 2025 Hz and 2225 Hz
- Window size = sample_rate / 300 (one bit period)
- Compare magnitudes: if |2225| > |2025|, bit = 1 (mark), else bit = 0 (space)

### 3. UART Framer

Recover byte boundaries from the raw bit stream:

- Detect start bit (transition from mark to space)
- Sample 8 data bits at the center of each bit period
- Verify 2 stop bits (both should be mark)
- Framing: 8N2 (no parity, 2 stop bits)
- Output: raw bytes

The NTP implementation uses a maximum-likelihood UART with 8 phase-shifted shift registers
for robust bit synchronization.

### 4. Burst Assembler

Collect 10 bytes into a complete burst and validate:

```rust
struct Burst {
    bytes: [u8; 10],
    second: u8,     // which second (31-39) this burst arrived
    format: Format, // A or B
}

enum Format { A, B }
```

- Detect burst boundaries using the 10 ms tick followed by mark tone
- Collect 10 characters
- Determine format: second 31 = Format B, seconds 32-39 = Format A
- Validate redundancy (identical for A, complemented for B)
- Nibble-swap all data bytes

### 5. Time Code Parser

Extract the decoded time from validated bursts:

```rust
struct ChuTime {
    year: u16,          // From Format B
    day_of_year: u16,   // From Format A (1-366)
    hour: u8,           // From Format A (0-23)
    minute: u8,         // From Format A (0-59)
    second: u8,         // From Format A (32-39)
    dut1: f32,          // From Format B (-0.8 to +0.8)
    tai_utc: u8,        // From Format B (leap seconds)
    dst_code: u8,       // From Format B (Canadian DST)
    leap_add: bool,     // From Format B control bits
    leap_sub: bool,     // From Format B control bits
}
```

### 6. State Machine

Track position within the minute to know what to expect:

```rust
enum DecoderState {
    WaitingForMinuteMarker,  // Looking for 500ms/1000ms tone
    CountingDUT1Ticks,       // Seconds 1-16, detecting split tones
    NormalTicks,             // Seconds 17-28
    ExpectingSilence,        // Second 29
    ExpectingFormatB,        // Second 31
    ReceivingFormatA(u8),    // Seconds 32-39 (tracking which one)
    NormalTicksLate,         // Seconds 40-49
    VoiceAnnouncement,       // Seconds 50-59
}
```

---

## Comparison with WWV and WWVB

| Feature               | CHU (Canada)                | WWV (USA)                    | WWVB (USA)                   |
|-----------------------|-----------------------------|------------------------------|------------------------------|
| **Operator**          | NRC Canada                  | NIST                         | NIST                         |
| **Location**          | Ottawa, ON                  | Fort Collins, CO             | Fort Collins, CO             |
| **Frequencies**       | 3.330, 7.850, 14.670 MHz   | 2.5, 5, 10, 15, 20, 25 MHz  | 60 kHz (longwave)            |
| **Modulation**        | H3E (USB + carrier)        | AM (DSB)                     | AM + PM (since 2012)         |
| **Time code method**  | FSK audio (Bell 103, 300 baud) | 100 Hz subcarrier (IRIG-H) | Pulse-width modulation       |
| **Data rate**         | 300 bps                     | 1 bps                        | 1 bps                        |
| **Time code format**  | BCD via 8N2 serial          | Modified IRIG-H BCD          | BCD via pulse widths          |
| **Mark/Space**        | 2225 / 2025 Hz (FSK)       | 100 Hz subcarrier on/off     | 17 dB power reduction         |
| **Frame length**      | ~367 ms (per burst)         | 1 minute                     | 1 minute                     |
| **Redundancy**        | 8 bursts/min (Format A)    | 1 frame/min                  | 1 frame/min                  |
| **Voice**             | Bilingual (EN/FR)           | Male (WWV) / Female (WWVH)   | None                         |
| **Tick tone**         | 1000 Hz, 300 ms             | 1000 Hz, 5 ms                | N/A                          |
| **Primary use case**  | NTP, radio clocks           | NTP, calibration             | Consumer "atomic" clocks     |
| **Power**             | 3-5 kW                     | 2.5-10 kW                   | 70 kW ERP                    |

### Key Differences for a Decoder Builder

1. **CHU is much faster to decode.** At 300 bps with 8 redundant bursts per minute, you can
   get a valid time reading in under 10 seconds. WWV and WWVB transmit at 1 bps and require
   a full minute for a complete frame.

2. **CHU uses standard serial encoding.** The Bell 103 FSK with 8N2 framing is essentially a
   modem protocol -- you are building a 300-baud modem decoder. WWV uses a subcarrier
   modulation scheme. WWVB uses power level changes.

3. **CHU's FSK tones are in a convenient range.** 2025/2225 Hz are easily resolved at typical
   audio sample rates (8 kHz or 48 kHz). A Goertzel filter or simple correlation works well.

4. **CHU provides more data per burst.** Each burst includes day-of-year, hour, minute, and
   second. One valid burst gives you the complete time.

5. **CHU is on shortwave (HF).** This means ionospheric propagation, fading, and the need
   for HF reception capability (direct sampling or upconverter with RTL-SDR). WWV is also HF.
   WWVB is longwave and requires different antenna/reception equipment.

---

## Time Accuracy

| Metric                           | Value                  |
|----------------------------------|------------------------|
| Broadcast accuracy               | 10^-4 seconds (100 us) |
| Carrier frequency accuracy       | < 5 x 10^-12          |
| International agreement          | Within 10 us           |
| Dominant error source (distant)  | Ionospheric path delay |

For a software decoder receiving locally in Ottawa, propagation delay is negligible (ground
wave at 15 km). The practical accuracy limit will be the sample rate and processing latency
of the SDR, not the broadcast accuracy.

---

## Practical Notes for the Decoder Demo

### Signal Acquisition

1. **Best frequency for Ottawa reception:** 7.850 MHz (strongest at 5 kW, least atmospheric
   interference for local reception)
2. **Fallback:** 3.330 MHz (good for nighttime, lower frequency propagates well)
3. **14.670 MHz:** Best for daytime long-distance, may have more noise locally

### Sample Rate Considerations

- The FSK tones are at 2025 and 2225 Hz, so Nyquist requires at least 4450 Hz sample rate
  for the audio
- NTP reference uses 8000 Hz audio sample rate
- RTL-SDR minimum sample rate is ~225 kHz -- you will need to decimate heavily
- Decimation chain: 225 kHz -> demodulate AM -> lowpass to ~3 kHz -> decimate to 8-48 kHz

### Tone Detection Approach

For the talk demo, a Goertzel-based approach is clean and explainable:

```rust
// Goertzel filter for detecting presence of a specific frequency
fn goertzel(samples: &[f32], target_freq: f32, sample_rate: f32) -> f32 {
    let k = (0.5 + (samples.len() as f32 * target_freq / sample_rate)) as usize;
    let w = 2.0 * std::f32::consts::PI * k as f32 / samples.len() as f32;
    let coeff = 2.0 * w.cos();

    let mut s0 = 0.0f32;
    let mut s1 = 0.0f32;
    let mut s2 = 0.0f32;

    for &sample in samples {
        s0 = sample + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }

    // Return magnitude squared
    s1 * s1 + s2 * s2 - coeff * s1 * s2
}

// Detect mark vs space
fn detect_bit(samples: &[f32], sample_rate: f32) -> bool {
    let mark_power = goertzel(samples, 2225.0, sample_rate);
    let space_power = goertzel(samples, 2025.0, sample_rate);
    mark_power > space_power  // true = mark (1), false = space (0)
}
```

### Synchronization Strategy

1. **Coarse sync:** Detect the 500 ms minute marker tone (or the silence at second 29)
2. **Fine sync:** The data burst ends precisely at 500 ms past the second
3. **Second counting:** Once synchronized, track seconds within the minute
4. **Burst alignment:** Look for the 10 ms tick followed by 123.3 ms mark tone as the
   start-of-burst indicator

---

## References

- [NRC Shortwave Station Broadcasts (CHU)](https://nrc.canada.ca/en/certifications-evaluations-standards/canadas-official-time/nrc-shortwave-station-broadcasts-chu) -- Official NRC documentation
- [CHU Broadcast Codes](https://nrc.canada.ca/en/certifications-evaluations-standards/canadas-official-time/chu-broadcast-codes) -- Official NRC BCD code format documentation
- [CHU (radio station) - Wikipedia](https://en.wikipedia.org/wiki/CHU_(radio_station)) -- General overview and history
- [Time and Frequency Station CHU (Canada)](https://www.eecis.udel.edu/~mills/ntp/chu.html) -- David L. Mills' detailed technical reference (NTP project)
- [Radio CHU Audio Demodulator/Decoder (NTP Driver 7)](https://www.ntp.org/documentation/drivers/driver7/) -- NTP reference implementation documentation
- [CHU - Signal Identification Wiki](https://www.sigidwiki.com/wiki/CHU) -- Signal characteristics and identification
- [WWV and WWVH Digital Time Code and Broadcast Format](https://www.nist.gov/pml/time-and-frequency-division/time-distribution/radio-station-wwv/wwv-and-wwvh-digital-time-code) -- NIST WWV documentation for comparison
- [WWVB - Wikipedia](https://en.wikipedia.org/wiki/WWVB) -- WWVB longwave time signal
- [Frequency Change for Canadian Time Transmission Station CHU](https://www.arrl.org/news/frequency-change-for-canadian-time-transmission-station-chu) -- ARRL article on 2009 frequency change
- [RTL-SDR Direct Sampling Mode](https://www.rtl-sdr.com/rtl-sdr-direct-sampling-mode/) -- HF reception with RTL-SDR
- [Bell 103 Modem - Wikipedia](https://en.wikipedia.org/wiki/Bell_103_modem) -- Bell 103 FSK standard
- [Bell 103 Modem Demodulator (GitHub)](https://github.com/jremington/Bell-103-modem-demodulator) -- Reference FSK demodulation implementation
