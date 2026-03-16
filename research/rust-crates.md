# Rust Crate Evaluation for SDR Work

Evaluation of the Rust crate ecosystem for software-defined radio, with a focus on
reliability for live conference demos. Research conducted 2026-02-16.

---

## Summary and Recommendations

| Layer | Recommended Crate | Confidence | Notes |
|---|---|---|---|
| SDR Hardware | `rtl-sdr-rs` | High | Active, V4 support, includes FM example |
| FM Demodulation | Roll your own | High | ~50 lines; `rtl-sdr-rs` example shows how |
| FFT | `rustfft` | High | Battle-tested, pure Rust, SIMD-accelerated |
| Audio Output | `cpal` | High | Industry standard, low-level control |
| Terminal UI | `ratatui` | High | Best-in-class TUI, massive community |
| Sample Types | `num-complex` | High | Standard complex number type |
| Audio DSP Utilities | `dasp` (optional) | Medium | Good for sample rate conversion if needed |

**Overall take**: The Rust SDR ecosystem is thin but workable. The hardware
interface layer (`rtl-sdr-rs`) is solid. For DSP, you are better off writing
the ~100 lines of FM demod and filtering yourself rather than depending on
half-maintained crates. The audio and TUI layers are excellent.

---

## SDR Hardware Interface

### `rtl-sdr-rs`

**Verdict: USE THIS. Best option for the demos.**

| Attribute | Value |
|---|---|
| Repository | [ccostes/rtl-sdr-rs](https://github.com/ccostes/rtl-sdr-rs) |
| crates.io | [rtl-sdr-rs](https://crates.io/crates/rtl-sdr-rs) |
| Version | 0.2.1 |
| Stars | ~130 |
| Forks | ~18 |
| Commits | 92 |
| License | GPL-2.0 |
| Open Issues | 7 |

**What it is**: A pure Rust port of the Osmocom `librtlsdr` C library, with
optional RTL-SDR Blog modifications (including V4 support via the R828D tuner
with 28.8 MHz upconverter).

**API overview**:
- `RtlSdr::open_first_available()` / `open_with_index(n)` / `open_with_serial(s)` — device opening
- `set_center_freq()`, `set_sample_rate()`, `set_tuner_gain()` — configuration
- `read_sync()` — blocking synchronous reads (simplest)
- `read_async()` — asynchronous callback-based reads (better for streaming)
- Supports device enumeration for multi-device setups

**RTL-SDR Blog V4 support**: Enabled via `--features rtl_sdr_blog` cargo
feature flag. This is critical since the V4 uses the R828D tuner which
requires driver modifications that differ from the standard R820T.

**FM example included**: The repo has a thorough
[`simple_fm.rs`](https://github.com/ccostes/rtl-sdr-rs/blob/main/examples/simple_fm.rs)
example that implements a complete FM receiver pipeline:
1. Two-thread architecture (SDR capture thread + demod/output thread)
2. 90-degree IQ rotation with optional NEON SIMD on ARM
3. Low-pass filter + decimation (capture rate -> demod rate)
4. FM discriminator using fast `atan2` approximation
5. Resampling from demod rate (170 kHz) to output rate (32 kHz)
6. Outputs raw s16 audio to stdout, piped to `play`

This example alone is worth more than most of the DSP crates listed below. It
proves the library works end-to-end for exactly our use case.

**Why it wins**: Pure Rust (no C library dependency at runtime), actively
maintained, V4 hardware support, includes a working FM demo that validates the
entire pipeline. The 130 stars indicate it is the de facto RTL-SDR crate.

**Risks**: GPL-2.0 license (fine for conference demos, would matter for
commercial use). Seven open issues suggest active use but also some rough
edges. Being a direct port of the C library means the API is functional but
not always idiomatic Rust.

**Demo suitability**: Excellent. The `simple_fm.rs` example is essentially
Demo 1 with minor modifications.


### `rtlsdr_mt`

**Verdict: SKIP. Abandoned since 2017.**

| Attribute | Value |
|---|---|
| Repository | [kchmck/rtlsdr_mt.rs](https://github.com/kchmck/rtlsdr_mt.rs) |
| crates.io | [rtlsdr_mt](https://crates.io/crates/rtlsdr_mt) |
| Version | 0.1.2 |
| Stars | ~26 |
| Forks | ~10 |
| Commits | 50 |
| Contributors | 2 |
| Last commit | February 2017 |
| Downloads/month | ~141 |
| License | GPL-2.0+ |

**What it is**: A high-level wrapper around `librtlsdr` that separates device
control from sample reading for multithreaded use. The `Controller` and
`Reader` are split into separate types so you can tune frequency in one thread
while reading samples in another.

**API overview**:
- `open(index)` returns `(Controller, Reader)`
- `Controller` — `enable_agc()`, `set_ppm()`, `set_center_freq()`
- `Reader` — `read_async(num_buffers, buf_len, callback)` with `&[u8]` in callback

**Why to skip**: Last commit was **2017**. No V4 support. Wraps the C
`librtlsdr` (FFI dependency). Only 26 stars. The threading model is nice in
theory but `rtl-sdr-rs` with a channel-based architecture achieves the same
thing with modern Rust patterns. The Rust 1.37.0 vintage means it predates
async/await, edition 2021, and many language improvements.

**Demo suitability**: Do not use. Hardware compatibility and build issues are
likely on modern systems. Not worth the risk for a live demo.


### `soapysdr`

**Verdict: SKIP for these demos. Over-engineered for our use case.**

| Attribute | Value |
|---|---|
| Repository | [kevinmehall/rust-soapysdr](https://github.com/kevinmehall/rust-soapysdr) |
| crates.io | [soapysdr](https://crates.io/crates/soapysdr) |
| Version | 0.4.5 |
| Stars | ~97 |
| Forks | ~31 |
| Commits | 137 |
| Last release | January 2, 2026 |
| License | Apache-2.0 / BSL-1.0 |
| Open Issues | 3 |

**What it is**: Rust bindings for the SoapySDR hardware abstraction layer,
which supports dozens of SDR devices (RTL-SDR, HackRF, USRP, LimeSDR,
BladeRF, Airspy, etc.).

**API overview**:
- Device enumeration and opening by driver/args
- `Device::rx_stream()` — create receive stream with configurable format
- Supports complex float format (CF32) compatible with tools like inspectrum
- Includes utility programs `soapy-sdr-info` and `soapy-sdr-stream`

**Why to skip for demos**: SoapySDR adds an entire abstraction layer we do
not need. We are using one specific device (RTL-SDR Blog V4) and `rtl-sdr-rs`
talks to it directly. SoapySDR requires installing the SoapySDR C library
plus the RTL-SDR SoapySDR module, adding two more points of failure to the
demo setup.

The README itself warns: *"many SoapySDR driver modules contain error
handling and thread safety bugs"* and the safe Rust wrappers *"assume drivers
meet SoapySDR's core API contract."* This is not the kind of caveat you want
for a live demo.

**When it would be useful**: If you were building a general-purpose SDR
application that needs to support multiple hardware backends. Not our case.

**Demo suitability**: Poor. Too many moving parts for a live demo with a
single RTL-SDR dongle.

---

## Signal Processing / DSP

### `demod_fm`

**Verdict: SKIP. Too simple to justify a dependency.**

| Attribute | Value |
|---|---|
| crates.io | [demod_fm](https://crates.io/crates/demod_fm) |
| docs.rs | [demod_fm](https://docs.rs/demod_fm/latest/demod_fm/) |
| Version | 1.0.0 |
| Stars | N/A (no separate GitHub repo for the library) |

**What it is**: A single-struct FM demodulator using phase difference
approximation.

**API overview**:
- `FmDemod::new(freq_deviation: f32, sample_rate: f32)` — creates demodulator
- `FmDemod::feed(sample: Complex32) -> f32` — processes one sample at a time
- Enforces Nyquist limit: `freq_deviation <= sample_rate / 2`

**Why to skip**: The entire crate is approximately 20-30 lines of actual
logic. It computes `arg(sample * conj(prev_sample)) * gain`. This is literally
one line of math:

```rust
let demod = (sample * prev.conj()).arg() * gain;
```

Adding a dependency for this is not justified, especially when `rtl-sdr-rs`'s
`simple_fm.rs` example implements a more complete FM demodulation pipeline
including the low-pass filter, decimation, and resampling stages that
`demod_fm` does NOT provide. You still need all those other stages for a
working FM receiver.

**Note on the GitHub repo**: The GitHub repository `cdeletre/demod_fm` is
actually a **command-line tool** (not the library crate), which is a fork of
`cubehub/demod` that wraps `liquid-dsp`. This is a different thing from the
crates.io `demod_fm` library. Confusing naming.

**Demo suitability**: Not needed. Write the discriminator inline — it is more
educational and gives you control over the implementation for the talk.


### `rust_radio` (johnwstanford)

**Verdict: SKIP. GPS-focused prototype, not useful for FM/general SDR.**

| Attribute | Value |
|---|---|
| Repository | [johnwstanford/rust_radio](https://github.com/johnwstanford/rust_radio) |
| Stars | ~2 |
| Forks | ~1 |
| Commits | 256 |
| Last tested Rust | 1.37.0 (2019) |

**What it is**: A DSP library inspired by GNU Radio and GNSS-SDR. Despite the
general name, it is almost entirely focused on GPS L1 C/A signal acquisition
and tracking.

**Why to skip**: Only 2 stars. Last tested with Rust 1.37.0 (2019 vintage).
The README explicitly says *"Everything is in the rapid prototyping phase and
there are not yet any guarantees on stability or backwards compatibility."*
The only working demo is GPS subframe decoding, which is irrelevant to FM
reception or CHU time signal decoding.

**Demo suitability**: Do not use. Unmaintained, wrong domain, likely does not
compile on modern Rust.


### `dasp` (RustAudio)

**Verdict: MAYBE USE for sample rate conversion. Do not depend on it heavily.**

| Attribute | Value |
|---|---|
| Repository | [RustAudio/dasp](https://github.com/RustAudio/dasp) |
| crates.io | [dasp](https://crates.io/crates/dasp) |
| Version | 0.11.0 |
| Stars | ~1,100 |
| Forks | ~74 |
| Commits | 358 |
| License | Apache-2.0 / MIT |

**What it is**: A modular suite of crates for PCM digital audio signal
processing. Formerly called `sample`. Provides fundamental types and traits
for working with audio samples, frames, and signals.

**Sub-crates** (13 total, pick what you need):
- `dasp_sample` — Sample trait, type conversions (u8, i16, f32, etc.)
- `dasp_frame` — Frame trait (mono, stereo, N-channel)
- `dasp_signal` — Iterator-like `Signal` trait for audio streams
- `dasp_interpolate` — Sample rate conversion (linear, sinc)
- `dasp_ring_buffer` — Fixed and bounded ring buffers
- `dasp_window` — Windowing functions (Hanning, Hamming, etc.)
- `dasp_peak` / `dasp_rms` / `dasp_envelope` — Signal analysis
- `dasp_graph` — Dynamic audio processing graph

**API design**: No dynamic allocations, no dependencies, `no_std` compatible.
The `Signal` trait provides an iterator-like interface:
```rust
let signal = dasp::signal::from_iter(samples.iter().cloned());
let resampled = signal.from_hz_to_hz(sinc_interpolator, 170_000.0, 32_000.0);
```

**Where it helps**: If you need clean sample rate conversion between the SDR
sample rate and the audio output rate, `dasp_interpolate` with a sinc
interpolator would be better quality than a simple decimation filter. The
`dasp_signal` abstractions are nice for composing processing pipelines.

**Where it does not help**: It has nothing SDR-specific. No FM demod, no
complex number support, no IQ processing. It operates in the audio domain,
not the RF domain.

**Demo suitability**: Moderate. Could simplify the resampling stage. But the
`simple_fm.rs` example in `rtl-sdr-rs` already implements resampling without
it. Use `dasp` only if you want cleaner abstractions for the audio output
stage.

---

## FFT

### `rustfft`

**Verdict: USE THIS if you need FFT (freq-scanner demo definitely needs it).**

| Attribute | Value |
|---|---|
| Repository | [ejmahler/RustFFT](https://github.com/ejmahler/RustFFT) |
| crates.io | [rustfft](https://crates.io/crates/rustfft) |
| Version | 6.4.1 |
| Stars | ~854 |
| Forks | ~57 |
| Commits | 457 |
| Last release | September 2025 |
| Min Rust | 1.61 |
| License | Apache-2.0 / MIT |

**What it is**: A high-performance, pure Rust FFT library. Computes FFTs of
any size (including prime sizes) in O(n log n) time with automatic SIMD
acceleration.

**API overview**:
```rust
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

let mut planner = FftPlanner::<f32>::new();
let fft = planner.plan_fft_forward(1024);

let mut buffer: Vec<Complex<f32>> = /* IQ samples */;
fft.process(&mut buffer);
// buffer now contains frequency-domain data
```

**SIMD support** (automatic, no special code needed):
- x86_64: AVX, FMA, SSE4.1
- AArch64: NEON
- WebAssembly: WASM SIMD (opt-in feature)

**Performance**: Claims to match or beat FFTW in benchmarks. The planner
automatically selects the fastest algorithm for the given size and CPU
features.

**Why it is essential**: The frequency scanner demo needs FFT to compute
power spectral density across frequency bins. `rustfft` is the clear winner
here — mature, fast, pure Rust, no C dependencies, widely used.

**Demo suitability**: Excellent. Rock-solid for computing spectrum displays in
the freq-scanner TUI.

### `phastft` (alternative)

Worth mentioning: [PhastFT](https://github.com/QuState/PhastFT) is a newer
"quantum-inspired" FFT in pure Rust that claims lower memory usage than
RustFFT. For a conference demo, stick with RustFFT — it has far more
community validation.

---

## Audio Output

### `cpal`

**Verdict: USE THIS. The right level of abstraction for real-time audio.**

| Attribute | Value |
|---|---|
| Repository | [RustAudio/cpal](https://github.com/RustAudio/cpal) |
| crates.io | [cpal](https://crates.io/crates/cpal) |
| Version | 0.17.2 |
| Stars | ~3,500 |
| Forks | ~477 |
| Contributors | 187 |
| Total downloads | ~8.7M |
| Last release | February 8, 2026 |
| Dependents | ~36,800 |
| License | Apache-2.0 |

**What it is**: Low-level, cross-platform audio I/O in pure Rust. This is the
de facto standard for audio in Rust.

**Platform support**:
- macOS: CoreAudio (+ optional JACK)
- Linux: ALSA (+ optional JACK)
- Windows: WASAPI (+ optional ASIO, JACK)
- iOS: CoreAudio
- Android: AAudio
- Web: WebAudio API

**API overview**:
```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

let host = cpal::default_host();
let device = host.default_output_device().unwrap();
let config = device.default_output_config().unwrap();

let stream = device.build_output_stream(
    &config.into(),
    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        // Fill `data` with audio samples from your demodulator
        for sample in data.iter_mut() {
            *sample = next_audio_sample();
        }
    },
    |err| eprintln!("Audio error: {}", err),
    None,
)?;
stream.play()?;
```

**Why it wins over rodio for our use case**: We are generating raw audio
samples from FM demodulation. `cpal` lets us write directly into the audio
buffer from a callback — zero overhead, zero format conversion, total control
over buffering and latency. This is exactly what you want for real-time SDR
audio.

**MSRV note**: Most backends require Rust 1.82+. Not an issue for us.

**Demo suitability**: Excellent. Direct buffer access means we can feed
demodulated samples straight to the speaker with minimal latency. The callback
model works naturally with a channel-based architecture where the SDR thread
sends samples and the audio callback consumes them.


### `rodio`

**Verdict: SKIP for the SDR demos. Use cpal directly.**

| Attribute | Value |
|---|---|
| Repository | [RustAudio/rodio](https://github.com/RustAudio/rodio) |
| crates.io | [rodio](https://crates.io/crates/rodio) |
| Version | 0.21.1 |
| Stars | ~2,300 |
| Forks | ~294 |
| Total downloads | ~5.3M |
| License | Apache-2.0 / MIT |

**What it is**: High-level audio playback library built on top of `cpal`.
Focuses on loading and playing audio files (MP3, FLAC, WAV, Vorbis) with
automatic mixing of multiple sound sources.

**API overview**:
```rust
use rodio::{Sink, OutputStream, Source};

let (_stream, handle) = OutputStream::try_default()?;
let sink = Sink::try_new(&handle)?;

// Custom source: implement the Source trait
// Source: Iterator<Item=f32> + channels() + sample_rate() + total_duration()
sink.append(my_custom_source);
```

**Custom sources**: You can implement the `Source` trait (which extends
`Iterator<Item=T>` where T is a sample type) to feed arbitrary sample data.
The `play_raw()` function accepts any source producing `f32` samples.

**Why to skip**: Rodio spawns a background thread for mixing and playback.
This adds latency and complexity compared to `cpal`'s direct callback model.
The file-format decoding (Symphonia, minimp3, etc.) is dead weight for us —
we are generating raw samples, not playing MP3s. Using `cpal` directly is
simpler and gives better control over the audio pipeline.

**When rodio makes sense**: Playing pre-recorded sound files, game audio with
multiple simultaneous sounds, applications where you do not need low-latency
sample-level control.

**Demo suitability**: Usable but suboptimal. The `Source` trait abstraction
adds a layer we do not need, and the background mixing thread introduces
latency that matters for real-time radio audio.

---

## Terminal UI

### `ratatui`

**Verdict: USE THIS. No competition in this space.**

| Attribute | Value |
|---|---|
| Repository | [ratatui/ratatui](https://github.com/ratatui/ratatui) |
| crates.io | [ratatui](https://crates.io/crates/ratatui) |
| Website | [ratatui.rs](https://ratatui.rs/) |
| Version | 0.30.0 |
| Stars | ~18,400 |
| Total downloads | ~17.4M |
| Dependents | 2,300+ crates |
| License | MIT |

**What it is**: The standard Rust TUI library. Successor to `tui-rs`. Provides
an immediate-mode rendering API with widgets for building terminal dashboards.

**Relevant widgets for freq-scanner**:
- `BarChart` — Signal strength per frequency
- `Sparkline` — Compact signal history
- `Canvas` — Custom drawing for spectrum waterfall
- `Table` — Active signal log
- `Gauge` — Signal level meters
- `Block` with borders/titles — Layout organization

**API overview** (v0.30):
```rust
use ratatui::prelude::*;

// Simplified main loop
ratatui::init();
loop {
    terminal.draw(|frame| {
        // Build your spectrum display
        let spectrum = BarChart::default()
            .data(&freq_power_data)
            .bar_width(1);
        frame.render_widget(spectrum, area);
    })?;
    // Handle input events
}
ratatui::restore();
```

**v0.30 changes**: Modular workspace (ratatui-core, ratatui-crossterm, etc.),
`no_std` support, new `ratatui::run()` convenience API. The `init()` /
`restore()` functions (added in v0.28) make setup trivial.

**Demo suitability**: Perfect. The freq-scanner TUI is exactly what ratatui
was built for. The spectrum display, signal log, and frequency info can all
be composed from built-in widgets. The 18k stars and massive community mean
any issue you hit will have answers. Thomas also has experience with ratatui
from the ctrlsys project.

---

## rtl_tcp Client Protocol

### No existing crate. Write it yourself.

**Verdict: Write a minimal client. The protocol is trivially simple.**

There is no published Rust crate that implements an `rtl_tcp` client. The only
Rust project found is [niclashoyer/rtltcp](https://github.com/niclashoyer/rtltcp),
which is a **server** reimplementation (replacing the C `rtl_tcp` binary), not
a client library.

**The protocol** ([K3XEC overview](https://k3xec.com/rtl-tcp/)):

1. **Connect**: TCP socket to `host:1234` (default port)
2. **Dongle info header**: Server sends a 12-byte struct on connect:
   - 4 bytes magic: `"RTL0"`
   - 4 bytes tuner type (u32 big-endian)
   - 4 bytes gain count (u32 big-endian)
3. **Commands** (client -> server): 5 bytes each:
   - 1 byte command ID
   - 4 bytes parameter (u32 big-endian)
   - Commands: `0x01` set freq, `0x02` set sample rate, `0x03` gain mode,
     `0x04` set gain, `0x05` freq correction, `0x08` AGC mode
4. **IQ stream** (server -> client): Continuous stream of interleaved u8 pairs
   (I, Q, I, Q, ...) where 128 = 0, 255 = +1, 0 = -1

A minimal Rust client is maybe 80 lines:

```rust
use std::io::{Read, Write};
use std::net::TcpStream;

struct RtlTcpClient {
    stream: TcpStream,
}

impl RtlTcpClient {
    fn connect(addr: &str) -> std::io::Result<Self> {
        let mut stream = TcpStream::connect(addr)?;
        let mut header = [0u8; 12];
        stream.read_exact(&mut header)?; // Read dongle info
        Ok(Self { stream })
    }

    fn send_command(&mut self, cmd: u8, param: u32) -> std::io::Result<()> {
        let mut buf = [0u8; 5];
        buf[0] = cmd;
        buf[1..5].copy_from_slice(&param.to_be_bytes());
        self.stream.write_all(&buf)
    }

    fn set_frequency(&mut self, hz: u32) -> std::io::Result<()> {
        self.send_command(0x01, hz)
    }

    fn set_sample_rate(&mut self, rate: u32) -> std::io::Result<()> {
        self.send_command(0x02, rate)
    }

    fn read_samples(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}
```

**Why write it yourself**: The protocol is dead simple. Adding an `rtl_tcp`
client gives you a fallback path for the demos — if the USB connection is
flaky on stage, you can run `rtl_tcp` on a Raspberry Pi backstage and connect
over the network. This also enables the Pi 4 B + OpenWebRX+ setup mentioned
in the README.

**Demo suitability**: High value, low effort. A safety net for live demos.

---

## Existing Rust SDR Projects Worth Studying

### `rustradio` (ThomasHabets)

| Attribute | Value |
|---|---|
| Repository | [ThomasHabets/rustradio](https://github.com/ThomasHabets/rustradio) |
| crates.io | [rustradio](https://crates.io/crates/rustradio) |
| Blog post | [RustRadio, and Roast My Rust](https://blog.habets.se/2023/10/RustRadio-Roast-My-Rust.html) |

A GNURadio-inspired framework with a block-based signal flow graph
architecture. Includes relevant blocks:
- **RtlSdrDecode** — Converts RTL-SDR u8 byte pairs to Complex I/Q
- **QuadratureDemod** — Core FM discriminator
- **IIR filter** — For de-emphasis filtering
- **Resampler** — Fractional rate conversion
- **Multiply by constant** — Gain adjustment

The FM receiver is presented as the "hello world" example. The RTL-SDR source
block requires enabling the `rtlsdr` feature and depends on `librtlsdr.so` at
runtime.

**Useful as**: Reference code for understanding block-based SDR architecture.
The blog post discusses Rust-specific design decisions. Not recommended as a
dependency — it adds framework overhead and the `librtlsdr.so` runtime
dependency that `rtl-sdr-rs` avoids.


### FutureSDR

| Attribute | Value |
|---|---|
| Repository | [FutureSDR/FutureSDR](https://github.com/FutureSDR/FutureSDR) |
| Stars | ~425 |
| Forks | ~71 |
| Commits | 1,278 |
| License | Apache-2.0 |

An async SDR runtime inspired by GNURadio, built with Tokio. Supports
Vulkan GPU acceleration and Xilinx FPGA interfacing. Includes example
applications for LoRa, M17, WLAN, ZigBee, and Rattlegram.

**Explicitly experimental**: The README warns *"it is likely that SDR
applications will require changes to the runtime"* and recommends cloning the
repo rather than adding it as a dependency.

**Useful as**: Inspiration for async SDR architecture. The block/flowgraph
design patterns are interesting. Too heavy and unstable for our demos.


### `rtl-sdr-rs` simple_fm.rs

Already discussed above, but worth calling out again: this single example file
is the most valuable reference in the entire Rust SDR ecosystem for our
specific use case. It implements the complete pipeline from USB dongle to
audio output in ~400 lines of well-commented Rust.

### eyrek/rust-radio

[eyrek/rust-radio](https://github.com/eyrek/rust-radio) — A simple FM radio
app for RTL-SDR that wraps `rtlsdr-rs` (note: different from `rtl-sdr-rs`)
and uses Rust bindings for `liquid-dsp` for demodulation. The `liquid-dsp`
dependency (C library) makes this less portable than a pure Rust approach.

---

## Architecture Recommendation for the Three Demos

### Demo 1: fm-receiver

```
RTL-SDR dongle
    |
    v
rtl-sdr-rs (read_async)
    |  raw u8 IQ pairs
    v
[Custom code] IQ rotation + low-pass filter + FM discriminator + resample
    |  f32 audio samples
    v
cpal (output stream callback)
    |
    v
Speakers
```

**Crates**: `rtl-sdr-rs`, `num-complex`, `cpal`
**Custom code**: ~150 lines of DSP (ported from `simple_fm.rs`)

### Demo 2: chu-decoder

```
RTL-SDR dongle (tuned to 3330/7850/14670 kHz)
    |
    v
rtl-sdr-rs (direct sampling mode for HF)
    |  raw u8 IQ pairs
    v
[Custom code] Narrowband FM demod + tone detection + BCD decode
    |  decoded time data
    v
Terminal output (println! or ratatui)
```

**Crates**: `rtl-sdr-rs`, `num-complex`, `rustfft` (for tone detection)
**Custom code**: CHU protocol decoder, BCD parser
**Note**: CHU is on shortwave (HF), which requires direct sampling mode on
the RTL-SDR V4. Verify `rtl-sdr-rs` supports `set_direct_sampling()`.

### Demo 3: freq-scanner

```
RTL-SDR dongle (sweeping frequencies)
    |
    v
rtl-sdr-rs (rapid retune + short captures)
    |  raw u8 IQ pairs per frequency step
    v
rustfft (compute power spectrum per step)
    |  frequency-domain power levels
    v
ratatui (spectrum display, signal log, waterfall)
    |
    v
Terminal
```

**Crates**: `rtl-sdr-rs`, `num-complex`, `rustfft`, `ratatui`, `crossterm`
**Custom code**: Sweep logic, signal detection thresholds, TUI layout

### Fallback: rtl_tcp client

For all three demos, a ~100-line `rtl_tcp` client module provides a network
fallback if USB is unreliable on stage. The client implements the same
`read_samples()` interface as the direct USB path, so the rest of the pipeline
does not change.

---

## Dependency Risk Assessment for Live Demos

| Risk | Mitigation |
|---|---|
| RTL-SDR USB connection drops | rtl_tcp network fallback via Pi |
| Audio device not found by cpal | Test on exact presentation laptop beforehand; have pre-recorded samples |
| rtl-sdr-rs does not support V4 feature X | The `rtl_sdr_blog` feature flag exists; test early with actual hardware |
| Compile issues on demo day | Pin all dependency versions in Cargo.lock; build release binary ahead of time |
| FFT too slow for real-time scanner | rustfft with SIMD is fast; use 1024-point FFT, not 65536 |
| No signals in venue | Pre-record IQ samples for offline playback mode |

The most important thing for a live demo: **have a pre-recorded fallback for
every demo**. Record IQ sample files that can be played back through the
exact same processing pipeline, minus the hardware.

---

## References

- [rtl-sdr-rs GitHub](https://github.com/ccostes/rtl-sdr-rs)
- [rtl-sdr-rs simple_fm.rs example](https://github.com/ccostes/rtl-sdr-rs/blob/main/examples/simple_fm.rs)
- [rtl-sdr-rs on crates.io](https://crates.io/crates/rtl-sdr-rs)
- [rtlsdr_mt GitHub](https://github.com/kchmck/rtlsdr_mt.rs)
- [rtlsdr_mt on lib.rs](https://lib.rs/crates/rtlsdr_mt)
- [rust-soapysdr GitHub](https://github.com/kevinmehall/rust-soapysdr)
- [soapysdr on crates.io](https://crates.io/crates/soapysdr)
- [demod_fm on docs.rs](https://docs.rs/demod_fm/latest/demod_fm/)
- [demod_fm source](https://docs.rs/demod_fm/1.0.0/src/demod_fm/lib.rs.html)
- [rust_radio GitHub](https://github.com/johnwstanford/rust_radio)
- [dasp GitHub](https://github.com/RustAudio/dasp)
- [RustFFT GitHub](https://github.com/ejmahler/RustFFT)
- [rustfft on crates.io](https://crates.io/crates/rustfft)
- [cpal GitHub](https://github.com/RustAudio/cpal)
- [cpal on crates.io](https://crates.io/crates/cpal)
- [rodio GitHub](https://github.com/RustAudio/rodio)
- [ratatui GitHub](https://github.com/ratatui/ratatui)
- [ratatui.rs](https://ratatui.rs/)
- [rustradio (ThomasHabets) GitHub](https://github.com/ThomasHabets/rustradio)
- [rustradio on docs.rs](https://docs.rs/rustradio/latest/rustradio/)
- [RustRadio, and Roast My Rust — blog post](https://blog.habets.se/2023/10/RustRadio-Roast-My-Rust.html)
- [FutureSDR GitHub](https://github.com/FutureSDR/FutureSDR)
- [FutureSDR documentation](https://www.futuresdr.org/learn/)
- [niclashoyer/rtltcp — Rust rtl_tcp server](https://github.com/niclashoyer/rtltcp)
- [K3XEC — Overview of the RTL TCP Protocol](https://k3xec.com/rtl-tcp/)
- [rtl_tcp protocol — DeepWiki](https://deepwiki.com/rtlsdrblog/rtl-sdr-blog/5.1-rtl_tcp-network-iq-streaming-server)
- [eyrek/rust-radio — FM Radio App](https://github.com/eyrek/rust-radio)
- [PhastFT GitHub](https://github.com/QuState/PhastFT)
- [RTL-SDR Blog V4 Users Guide](https://www.rtl-sdr.com/v4/)
