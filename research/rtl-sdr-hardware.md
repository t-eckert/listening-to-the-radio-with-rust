# RTL-SDR Hardware and rtl_tcp Protocol

Research notes for "Listening to the Radio with Rust" talk. Covers the hardware architecture
of RTL-SDR dongles, the rtl_tcp network protocol, and the RTL-SDR Blog V4 improvements.

---

## Hardware Architecture

### Overview

An RTL-SDR dongle contains two main ICs:

1. **R820T2 (or R828D) tuner** -- receives RF from the antenna, downconverts to intermediate
   frequency (IF) or zero-IF
2. **RTL2832U demodulator** -- digitizes the IF signal via an internal ADC, packages
   the samples, and sends them over USB

Originally designed as a DVB-T (digital TV) receiver, the RTL2832U was repurposed for SDR
in ~2012 when developers discovered they could access the raw ADC samples by bypassing the
demodulator's TV-specific firmware.

### R820T2 Tuner

The R820T2 (Rafael Micro, Taiwan) is a highly integrated silicon tuner. It includes:

- **Low Noise Amplifier (LNA)** -- first gain stage, most important for SNR
- **Mixer** -- downconverts RF to IF using a local oscillator
- **Fractional PLL** -- generates the local oscillator frequency with fine resolution
- **Variable Gain Amplifier (VGA)** -- final gain stage before output to the demodulator
- **Tracking filter** -- provides some front-end selectivity
- **Voltage regulator** -- integrated power management

**Key specs:**
- Frequency range: **24 MHz to 1.766 GHz** (the "HF gap" below 24 MHz requires direct
  sampling or an upconverter)
- Supply voltage: 3.3V
- Current draw: < 178 mA
- Interface to RTL2832U: **I2C** for register control, analog IF output for signal path
- Supported IF modes: 36.125 MHz (standard IF), 4.57 MHz (low IF), or zero-IF

**Gain control:**

The R820T2 has three independent gain stages controllable via I2C registers:

| Stage | Role | Notes |
|-------|------|-------|
| LNA | First amplification | Most important for SNR |
| Mixer | Downconversion gain | Secondary importance |
| VGA | Final amplification | Least important -- adds thermal noise |

In "manual gain" mode, librtlsdr exposes 29 composite gain values (in tenths of dB):

```
0.0  0.9  1.4  2.7  3.7  7.7  8.7  12.5  14.4  15.7
16.6 19.7 20.7 22.9 25.4 28.0 29.7 32.8  33.8  36.4
37.2 38.6 40.2 42.1 43.4 43.9 44.5 48.0  49.6
```

Note: 43.9 dB and 44.5 dB are reported to have identical actual gain.

The RTL2832U acts as an I2C master to the tuner. To send gain commands, the I2C repeater
in the RTL2832U must be enabled first so commands are forwarded to the tuner IC.

### RTL2832U Demodulator

The RTL2832U (Realtek Semiconductor) is a DVB-T COFDM demodulator with a USB 2.0 interface.
For SDR use, its TV demodulation is bypassed and we access the raw digitized samples.

**Internal signal path (SDR mode):**

1. Analog IF signal arrives from the tuner
2. **8-bit ADC** samples the signal at the configured rate
3. **Digital Down-Converter (DDC)** -- I/Q mixers (90 degrees apart) shift from IF to baseband
4. **Digital low-pass filter** -- removes out-of-band content
5. **Resampler** -- adjusts sample rate (controlled by `rsamp_ratio` register)
6. **8-bit I/Q output** -- interleaved I and Q bytes are sent over USB

**Key specs:**
- ADC resolution: **8 bits** (this is the fundamental dynamic range limitation)
- Reference crystal: **28.8 MHz**
- Supported IF frequencies: 36.125 MHz, 4.57 MHz, or zero-IF
- USB interface: USB 2.0 High Speed (480 Mbps theoretical max)
- Package: 48-pin QFN

### 8-Bit ADC: Implications

The 8-bit ADC is the single biggest limitation of RTL-SDR hardware. Understanding its
implications is important for writing good Rust code that processes RTL-SDR data.

**Dynamic range:** ~48 dB theoretical (6 dB per bit), but effective dynamic range is
closer to **~40 dB** after real-world noise and imperfections.

**Quantization noise:** With only 256 discrete levels, weak signals near strong ones
can be lost in quantization noise. The voltage resolution is approximately 400 uV,
compared to ~10 uV on higher-resolution receivers like the Airspy (12-bit).

**Data representation:**
- Each sample is an **unsigned 8-bit integer** (uint8)
- Value range: 0 to 255
- Midpoint (zero signal): **128**
- Mapping: 0 = -1.0 (full negative), 128 = 0.0 (zero), 255 = +1.0 (full positive)

**For Rust code**, the conversion from raw bytes to floating point samples:

```rust
// Convert a raw u8 sample to f32 in range [-1.0, 1.0]
fn sample_to_f32(raw: u8) -> f32 {
    (raw as f32 - 128.0) / 128.0
}

// Or for an IQ pair
struct IqSample {
    i: f32,
    q: f32,
}

fn parse_iq(i_raw: u8, q_raw: u8) -> IqSample {
    IqSample {
        i: (i_raw as f32 - 128.0) / 128.0,
        q: (q_raw as f32 - 128.0) / 128.0,
    }
}
```

**Processing gain through decimation:** The low ADC resolution can be partially compensated
by oversampling and decimating. Sampling at a higher rate than needed and then digitally
filtering/decimating removes out-of-band quantization noise, effectively increasing the
SNR within the band of interest. Each factor-of-4 decimation yields roughly 6 dB of
processing gain (equivalent to one extra bit of resolution).

### Sample Rate

The RTL2832U supports sample rates from **225.001 kHz to 3.2 MHz** (in samples per second,
where each "sample" is an I/Q pair).

**Practical limits:**

| Rate | Status | Notes |
|------|--------|-------|
| 0.225 - 0.300 MHz | Low | Minimum useful range |
| 0.9 - 2.4 MHz | Recommended | Sweet spot for most applications |
| 2.4 MHz | Common default | Good balance of bandwidth and stability |
| 2.56 MHz | Also common | Power-of-two friendly |
| 2.8 MHz | Tested max | Highest rate confirmed without sample drops |
| 3.2 MHz | Hardware max | Often drops samples due to USB throughput limits |

**Data throughput calculation:**
- At 2.4 MS/s: 2,400,000 samples/sec x 2 bytes (I+Q) = **4.8 MB/s**
- At 3.2 MS/s: 3,200,000 samples/sec x 2 bytes (I+Q) = **6.4 MB/s**
- USB 2.0 High Speed theoretical: 480 Mbps = 60 MB/s (plenty of headroom)
- In practice, USB overhead and scheduling reduce effective throughput

### USB Interface

The RTL2832U communicates over USB 2.0 High Speed using **bulk transfers** on endpoint
**0x81** (IN direction).

**librtlsdr buffer architecture:**

The library uses an asynchronous streaming model with pre-allocated ring buffers:

| Parameter | Default | Notes |
|-----------|---------|-------|
| Buffer count | 15 | Number of in-flight USB transfers |
| Buffer size | 262,144 bytes (256 KB) | Each buffer |
| Total buffered | 3,932,160 bytes (~3.75 MB) | All buffers combined |
| Buffer latency at 2.048 MS/s | ~46 ms | Time to fill all buffers |
| Buffer latency at 2.4 MS/s | ~39 ms | At common default rate |

**Buffer size constraints:**
- Must be a multiple of **512 bytes** (USB packet size)
- Should be a multiple of **16,384 bytes** (USB Request Block / URB size) for efficiency
- Default: `16 * 32 * 512 = 262,144 bytes`

**Streaming flow:**

1. Library pre-allocates `buf_num` transfer buffers of `buf_len` bytes each
2. All buffers are submitted to libusb as bulk transfer requests on endpoint 0x81
3. As each transfer completes, the user callback is invoked with the buffer contents
4. The buffer is immediately resubmitted for the next transfer
5. Multiple transfers are always in-flight simultaneously to prevent gaps

**For Rust code**, the key function signature in librtlsdr:

```c
// C API -- what Rust FFI bindings wrap
int rtlsdr_read_async(
    rtlsdr_dev_t *dev,
    rtlsdr_read_async_cb_t cb,  // callback: fn(buf: *u8, len: u32, ctx: *void)
    void *ctx,                   // user context passed to callback
    uint32_t buf_num,            // 0 = default (15)
    uint32_t buf_len             // 0 = default (256 KB), must be multiple of 512
);
```

In Rust, this typically becomes a closure or channel-based pattern where the callback
pushes buffers into an `mpsc::channel` or `crossbeam::channel` for processing on
another thread.

### Direct Sampling Mode (V3 and earlier)

For receiving frequencies **below 24 MHz** (HF/shortwave), the tuner cannot be used
because its minimum frequency is 24 MHz. Direct sampling bypasses the tuner entirely.

**How it works:**

The RTL2832U has two ADC inputs (I-branch and Q-branch) that normally receive the
tuner's analog IF output. In direct sampling mode, RF is routed directly to one of
these ADC inputs:

- **Mode 1 (I-branch):** Direct sampling on the I ADC input
- **Mode 2 (Q-branch):** Direct sampling on the Q ADC input -- this is the one used
  by the RTL-SDR Blog V3, where the Q input is connected to the antenna port through
  the bias tee network

**Frequency coverage in direct sampling:**
- DC to 14.4 MHz (Nyquist limit of the 28.8 MHz crystal)
- Signals above 14.4 MHz will **alias/fold** around 14.4 MHz (Nyquist folding)
- This folding is a significant problem -- a signal at 20 MHz appears at 8.8 MHz

**Relevance to the CHU demo:**
- CHU transmits on **3.330 MHz**, **7.850 MHz**, and **14.670 MHz**
- 3.330 and 7.850 MHz are well within the direct sampling range
- 14.670 MHz is just above the 14.4 MHz Nyquist limit -- marginal with direct sampling
- The V4's upconverter approach (see below) handles all three cleanly

**Software control:**
- In librtlsdr: `rtlsdr_set_direct_sampling(dev, mode)` where mode = 0 (off), 1 (I), 2 (Q)
- In rtl_tcp: command `0x09` with parameter 0, 1, or 2
- The RTL-SDR Blog fork auto-switches to direct sampling below 24 MHz for V3 dongles

---

## rtl_tcp Network Protocol

### Overview

`rtl_tcp` is a lightweight TCP server that exposes an RTL-SDR dongle over the network.
It runs on the machine physically connected to the dongle (e.g., a Raspberry Pi) and
streams raw IQ samples to any TCP client.

**Use case for the talk:** Run rtl_tcp on a Raspberry Pi with the dongle, connect from
a laptop running the Rust demos. This separates the USB driver dependency from the
demo code and makes the architecture cleaner to explain.

**Default port:** 1234

**Starting the server:**
```bash
rtl_tcp -a 0.0.0.0 -p 1234 -s 2400000 -f 100100000
#        ^listen addr ^port  ^sample rate ^frequency (100.1 MHz)
```

### Connection Protocol

The protocol is simple and stateless. All multi-byte integers are in **network byte order
(big-endian)**.

#### Step 1: Connect

Open a TCP socket to the server. No handshake, no authentication.

#### Step 2: Receive Dongle Info (12 bytes)

Immediately after connection, the server sends a 12-byte header:

```
Offset  Size  Type     Field        Description
------  ----  ------   ----------   ----------------------------------
0       4     char[4]  magic        Always "RTL0" (0x52 0x54 0x4C 0x30)
4       4     uint32   tuner_type   Tuner IC type (big-endian)
8       4     uint32   gain_count   Number of valid gain values (big-endian)
```

**Tuner type values:**

| Value | Tuner | Notes |
|-------|-------|-------|
| 0 | Unknown | |
| 1 | E4000 | Older dongles, discontinued |
| 2 | FC0012 | |
| 3 | FC0013 | |
| 4 | FC2580 | |
| 5 | R820T | Also covers R820T2 (RTL-SDR Blog V3) |
| 6 | R828D | RTL-SDR Blog V4 |

**Rust struct for parsing:**

```rust
#[derive(Debug)]
struct DongleInfo {
    magic: [u8; 4],       // Should be b"RTL0"
    tuner_type: u32,      // See table above
    gain_count: u32,      // Number of valid gain settings
}

impl DongleInfo {
    fn from_bytes(buf: &[u8; 12]) -> Self {
        DongleInfo {
            magic: [buf[0], buf[1], buf[2], buf[3]],
            tuner_type: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            gain_count: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
        }
    }

    fn is_valid(&self) -> bool {
        &self.magic == b"RTL0"
    }
}
```

#### Step 3: Receive IQ Data Stream

After the 12-byte header, the server continuously streams raw IQ data:

```
[I0][Q0][I1][Q1][I2][Q2]...
```

- Each value is a **uint8** (unsigned 8-bit integer)
- Samples are interleaved: I, Q, I, Q, I, Q, ...
- Value 128 = zero, 0 = -1, 255 = +1
- Data flows at the configured sample rate until the socket is closed
- No framing, no packet boundaries -- it is a continuous byte stream

**Rust reading pattern:**

```rust
use std::io::Read;
use std::net::TcpStream;

fn read_iq_samples(stream: &mut TcpStream, buf: &mut [u8]) -> std::io::Result<usize> {
    // Read raw bytes -- will get as many as available
    let bytes_read = stream.read(buf)?;

    // Ensure we have an even number of bytes (complete I/Q pairs)
    Ok(bytes_read & !1) // Round down to even
}
```

#### Step 4: Send Commands (5 bytes each)

Commands are sent from client to server at any time. Each command is exactly 5 bytes:

```
Offset  Size  Type    Field      Description
------  ----  ------  ---------  ----------------------------------
0       1     uint8   command    Command ID (see table below)
1       4     uint32  parameter  Command parameter (big-endian)
```

**Command table:**

| ID | Command | Parameter | Description |
|----|---------|-----------|-------------|
| 0x01 | Set frequency | Hz (uint32) | Center frequency in Hz |
| 0x02 | Set sample rate | S/s (uint32) | Sample rate in samples per second |
| 0x03 | Set gain mode | 0=AGC, 1=manual | Tuner gain mode |
| 0x04 | Set gain | tenths of dB | Tuner gain (e.g., 496 = 49.6 dB) |
| 0x05 | Set freq correction | ppm (int32) | Frequency correction in PPM |
| 0x06 | Set IF gain | stage/gain | IF stage gain (stage in upper 16 bits, gain in lower 16) |
| 0x07 | Set test mode | 0=off, 1=on | Enable test mode (counter instead of samples) |
| 0x08 | Set AGC mode | 0=off, 1=on | RTL2832U internal AGC |
| 0x09 | Set direct sampling | 0=off, 1=I, 2=Q | Direct sampling mode (for HF) |
| 0x0A | Set offset tuning | 0=off, 1=on | Offset tuning mode |
| 0x0B | Set RTL xtal freq | Hz (uint32) | RTL2832U crystal frequency |
| 0x0C | Set tuner xtal freq | Hz (uint32) | Tuner crystal frequency |
| 0x0D | Set gain by index | index (uint32) | Set gain using gain table index |
| 0x0E | Set bias tee | 0=off, 1=on | Enable/disable bias tee power (V3/V4) |

**Rust command builder:**

```rust
#[repr(u8)]
enum RtlTcpCommand {
    SetFrequency = 0x01,
    SetSampleRate = 0x02,
    SetGainMode = 0x03,
    SetGain = 0x04,
    SetFreqCorrection = 0x05,
    SetIfGain = 0x06,
    SetTestMode = 0x07,
    SetAgcMode = 0x08,
    SetDirectSampling = 0x09,
    SetOffsetTuning = 0x0A,
    SetRtlXtalFreq = 0x0B,
    SetTunerXtalFreq = 0x0C,
    SetGainByIndex = 0x0D,
    SetBiasTee = 0x0E,
}

fn build_command(cmd: RtlTcpCommand, param: u32) -> [u8; 5] {
    let param_bytes = param.to_be_bytes();
    [cmd as u8, param_bytes[0], param_bytes[1], param_bytes[2], param_bytes[3]]
}

// Example: tune to 100.1 MHz FM
let cmd = build_command(RtlTcpCommand::SetFrequency, 100_100_000);
stream.write_all(&cmd)?;
```

### Typical Client Session

A complete rtl_tcp client session in pseudocode:

```
1. Connect to server (TCP)
2. Read 12 bytes -> parse DongleInfo, verify magic == "RTL0"
3. Send command: set sample rate (e.g., 2,400,000)
4. Send command: set gain mode (1 = manual)
5. Send command: set gain (e.g., 496 = 49.6 dB)
6. Send command: set frequency (e.g., 100,100,000 = 100.1 MHz)
7. Loop: read IQ byte stream, process in chunks
8. Close socket to disconnect
```

### Protocol Quirks and Notes

- **No acknowledgment:** Commands are fire-and-forget. The server does not respond
  to commands -- it just changes the dongle state.
- **No error reporting:** If a command fails (e.g., invalid frequency), the server
  silently ignores it. The data stream continues.
- **No framing on the data stream:** IQ data is a continuous byte stream with no
  delimiters. TCP handles delivery order, but you must handle partial reads.
- **Latency:** Network latency adds to USB latency. On a local network (Raspberry Pi),
  expect 1-5 ms additional. This is fine for non-real-time applications.
- **Multiple clients:** Standard rtl_tcp supports only **one client** at a time.
- **Forks exist:** Various forks add custom commands (e.g., rtl_tcp_iq). The
  standard command set above (0x01-0x0E) is supported by all versions.
- **Byte order:** The initial search results mention "little-endian" but this is
  incorrect. The protocol uses **big-endian (network byte order)** for all multi-byte
  integers, as confirmed by the source code and multiple implementations.

---

## RTL-SDR Blog V4 Differences

The RTL-SDR Blog V4 is a significant redesign compared to generic RTL-SDR dongles
and the earlier V3.

### Hardware Changes

| Feature | Generic / V3 | V4 |
|---------|-------------|-----|
| Tuner IC | R820T2 / R860 | **R828D** |
| HF reception | Direct sampling (Q-branch ADC) | **Built-in upconverter (SA612 mixer)** |
| Input filtering | None | **Triplexer (HF/VHF/UHF)** |
| Interference rejection | None | **Notch filters (AM/FM/DAB broadcast)** |
| Crystal | 1 PPM TCXO | 1 PPM TCXO |
| Bias tee | 4.5V, software-activated | 4.5V, software-activated, up to 180 mA |
| USB connector | Micro-USB (V3) | **USB-C available** |
| Case | Aluminum with passive cooling | Aluminum with passive cooling |
| Phase noise | Standard | **Improved (better power supply design)** |
| Power draw | Standard | **Slightly lower** |
| Heat | Standard | **Slightly less** |

### The Built-In HF Upconverter

This is the most important V4 change for the talk, especially for the CHU demo.

**V3 approach (direct sampling):**
- Bypasses the tuner, feeds RF directly into the RTL2832U's Q-branch ADC
- Limited to 0-14.4 MHz (Nyquist of 28.8 MHz crystal)
- Signals fold around 14.4 MHz -- a signal at 20 MHz aliases to 8.8 MHz
- No tuner gain available on HF -- sensitivity depends entirely on the ADC
- Works but has significant limitations

**V4 approach (upconverter):**
- Uses a **SA612 double-balanced mixer** to shift HF signals up by 28.8 MHz
- The upconverted signal is then received by the R828D tuner normally
- No Nyquist folding -- the entire 0-28 MHz HF band is usable without aliasing
- **Tuner gain is available on HF** -- significant sensitivity improvement
- The mixer uses the same 28.8 MHz oscillator already present for the RTL2832U/tuner

**What this means for CHU reception:**
- CHU 3.330 MHz -> upconverted to 32.130 MHz -> tuned normally by R828D
- CHU 7.850 MHz -> upconverted to 36.650 MHz -> tuned normally by R828D
- CHU 14.670 MHz -> upconverted to 43.470 MHz -> tuned normally by R828D
- All three CHU frequencies work cleanly with the V4, with tuner gain

### Triplexer

The V4 splits the SMA antenna input into three frequency bands:

| Band | Range | Connected to |
|------|-------|-------------|
| HF | 0 - 28 MHz | SA612 upconverter -> R828D input 1 |
| VHF | 28 - 250 MHz | R828D input 2 |
| UHF+ | 250 MHz - 1.766 GHz | R828D input 3 |

The R828D tuner has three separate inputs (an improvement over the R820T2's single
input), enabling this triplexing. The triplexer provides **28-43 dB of out-of-band
isolation**, meaning strong broadcast stations in one band are much less likely to
cause interference or desensitization when receiving in another band.

### Notch Filters

The R828D has an open-drain output pin that the V4 uses to switch in simple notch
filters for common interference sources:

- **Broadcast AM** (MW band)
- **Broadcast FM** (88-108 MHz)
- **DAB** (digital audio broadcast, ~174-240 MHz)

These provide an additional **5-10 dB of rejection** on top of the triplexer isolation.

### Software/Driver Requirements

The V4 requires the **RTL-SDR Blog fork of librtlsdr** (not the standard Osmocom drivers).
This fork handles:

- Automatic detection of V4 hardware via EEPROM
- Transparent HF upconverter frequency translation (you tune to 3.33 MHz and the driver
  automatically adds the 28.8 MHz offset)
- Bias tee control
- Automatic switching between tuner inputs based on frequency band
- For V3 dongles, automatic direct sampling mode switching below 24 MHz

**GitHub:** [rtlsdrblog/rtl-sdr-blog](https://github.com/rtlsdrblog/rtl-sdr-blog)

**For Rust crates:** Any Rust crate that wraps librtlsdr needs to link against the
RTL-SDR Blog fork to support V4 features. If using rtl_tcp, the server must be
built from the Blog fork.

---

## Practical Notes for Rust Development

### Connecting via rtl_tcp (Recommended for the Talk)

Using rtl_tcp is simpler than direct USB access for demos:

- No need for librtlsdr bindings or USB driver setup on the demo machine
- The protocol is trivial to implement in pure Rust (TCP socket + byte parsing)
- Separates hardware concerns from signal processing code
- Can pre-start the server on a Raspberry Pi, just connect from the laptop

### Architecture for the Demos

```
[RTL-SDR Dongle] --USB--> [Raspberry Pi running rtl_tcp]
                                      |
                                   TCP/IP
                                      |
                              [Laptop running Rust demos]
                                      |
                           +----------+----------+
                           |          |          |
                      FM Receiver  CHU Decoder  Scanner
```

### Buffer Management in Rust

When reading from rtl_tcp, TCP gives you arbitrary-sized chunks. You need to:

1. Maintain a read buffer that accumulates bytes
2. Process complete I/Q pairs (every 2 bytes)
3. Handle partial reads at the end of buffers

```rust
// Example: reading from rtl_tcp into a processing buffer
let mut tcp_buf = [0u8; 65536]; // 64 KB read buffer
let mut remainder: Option<u8> = None; // Leftover byte from previous read

loop {
    let n = stream.read(&mut tcp_buf)?;
    if n == 0 { break; } // Connection closed

    let mut offset = 0;

    // Handle leftover byte from previous read
    if let Some(i_byte) = remainder.take() {
        process_iq_sample(i_byte, tcp_buf[0]);
        offset = 1;
    }

    // Process complete pairs
    while offset + 1 < n {
        process_iq_sample(tcp_buf[offset], tcp_buf[offset + 1]);
        offset += 2;
    }

    // Save leftover byte if odd number of bytes read
    if offset < n {
        remainder = Some(tcp_buf[offset]);
    }
}
```

### Key Frequencies for Demos

| Signal | Frequency | Mode | Notes |
|--------|-----------|------|-------|
| FM broadcast | 88-108 MHz | Wideband FM | Any local station works |
| CHU time signal | 3.330 MHz | AM/SSB | Requires HF capability |
| CHU time signal | 7.850 MHz | AM/SSB | Requires HF capability |
| CHU time signal | 14.670 MHz | AM/SSB | Requires HF capability |
| Aviation (YOW) | 118-137 MHz | AM | Ottawa airport |
| NOAA weather | 137.1 MHz | APT (FM) | Satellite passes |
| ADS-B | 1090 MHz | Mode S | Aircraft transponders |

---

## References

- [RTL2832U + R820T2 Tuner Overview](https://www.petervis.com/Digital%20Terrestrial%20Receivers/dvb-t+fm+dab-820t2-and-sdr/rtl2832u-r820t2-tuner.html) -- tuner architecture and integration
- [RTL2832U Datasheet](https://homepages.uni-regensburg.de/~erc24492/SDR/Data_rtl2832u.pdf) -- Realtek official datasheet (V1.4, confidential but widely circulated)
- [RTL2832U: The Mystery Chip at the Heart of RTL-SDR](https://homepages.uni-regensburg.de/~erc24492/SDR/RTL2832U.pdf) -- detailed reverse-engineering analysis
- [Overview of the RTL TCP Protocol (K3XEC)](https://k3xec.com/rtl-tcp/) -- protocol documentation with byte layouts
- [Overview of the RTL TCP Protocol (hz.tools)](https://hz.tools/rtl_tcp/) -- alternative protocol reference
- [rtl_tcp protocol specification (hayguen/librtlsdr)](https://github.com/hayguen/librtlsdr_non_fork_master/blob/master/protocol_rtl_tcp.txt) -- protocol spec in text format
- [rtl_tcp.c source (rtlsdrblog fork)](https://github.com/rtlsdrblog/rtl-sdr-blog/blob/master/src/rtl_tcp.c) -- authoritative server implementation
- [librtlsdr source (osmocom)](https://github.com/osmocom/rtl-sdr/blob/master/src/librtlsdr.c) -- reference library implementation
- [rtl-sdr-blog driver fork (GitHub)](https://github.com/rtlsdrblog/rtl-sdr-blog) -- modified drivers for V3/V4
- [Asynchronous Data Streaming (DeepWiki)](https://deepwiki.com/rtlsdrblog/rtl-sdr-blog/3.4-asynchronous-data-streaming) -- buffer architecture analysis
- [Direct Sampling Mode (DeepWiki)](https://deepwiki.com/rtlsdrblog/rtl-sdr-blog/7.1-direct-sampling-mode) -- direct sampling technical details
- [RTL-SDR Direct Sampling Mode](https://www.rtl-sdr.com/rtl-sdr-direct-sampling-mode/) -- practical guide to direct sampling
- [RTL-SDR Blog V4 Dongle Initial Release](https://www.rtl-sdr.com/rtl-sdr-blog-v4-dongle-initial-release/) -- V4 announcement with technical details
- [RTL-SDR Blog V4 Datasheet (PDF)](https://www.rtl-sdr.com/wp-content/uploads/2024/12/RTLSDR_V4_Datasheet_V_1_0.pdf) -- official V4 datasheet
- [RTL-SDR Blog V4 Users Guide](https://www.rtl-sdr.com/v4/) -- setup and usage guide
- [RTL-SDR Blog V4 Review (Elektor)](https://www.elektormagazine.com/review/rtl-sdr-blog-v4-better-than-v3-review) -- independent review with measurements
- [R820T Gain Accuracy Tests (SimonsDialogs)](https://www.simonsdialogs.com/2014/09/r820t-rtl2832u-sdr-usb-stick-gain-accuracy-tests/) -- measured gain table values
- [New R820T Driver with LNA/Mixer/VGA Gain Settings](https://www.rtl-sdr.com/new-r820t-driver-lnamixervga-gain-settings/) -- per-stage gain control
- [Gain Control Functions (DeepWiki)](https://deepwiki.com/rtlsdrblog/rtl-sdr-blog/6.3-gain-control-functions) -- gain control API analysis
- [SDR Receiver ADC Performance](https://play.fallows.ca/wp/radio/software-defined-radio/sdr-receiver-adc-performance/) -- 8-bit ADC limitations and dynamic range
- [About RTL-SDR](https://www.rtl-sdr.com/about-rtl-sdr/) -- general overview and history
- [PySDR: RTL-SDR in Python](https://pysdr.org/content/rtlsdr.html) -- sample rate and ADC details
- [rtltcp Go Package](https://pkg.go.dev/hz.tools/sdr/rtltcp) -- Go implementation reference (useful for protocol verification)
- [bemasher/rtltcp (Go)](https://github.com/bemasher/rtltcp/blob/master/rtltcp.go) -- another Go implementation with dongle info parsing
