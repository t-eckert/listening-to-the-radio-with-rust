# sdr

**Not a demo — the shared library the multi-file demos are built on.**

If you're here to read one file that does the whole job, read
[`fm-single`](../fm-single/) instead. This crate is what that file looks like once the
pieces have been named and given homes.

```rust
use sdr::{open_source, SdrSource, SourceConfig};
use sdr::dsp::{fm::FmDemodulator, filter::LowPassFilter};
```

## Layout

### `source/` — where samples come from

Everything implements the `SdrSource` trait, so a demo doesn't care which one it got:

| Source | Argument | Notes |
|---|---|---|
| `FileSource` | `file:PATH` | Replay a recorded `.iq` capture |
| `TcpSource` | `tcp:HOST:PORT` | Read from `rtl_tcp`, local or remote |
| `UsbSource` | `usb[:INDEX]` | Direct USB, behind the `usb` feature |

```rust
pub trait SdrSource: Send {
    fn read(&mut self, buf: &mut [IqSample]) -> Result<usize>;
    fn set_frequency(&mut self, hz: u32) -> Result<()>;
    // ...
}
```

`parse_source_arg` turns those `file:` / `tcp:` / `usb:` strings into a source, which is
why every demo takes the same `--source` flag.

### `dsp/` — what happens to the samples

| Module | What it does |
|---|---|
| `fm` | FM demodulation — the angle between consecutive samples |
| `am` | AM demodulation — distance from the origin |
| `filter` | Low-pass filtering and decimation |
| `deemphasis` | Undoes the transmitter's treble boost (75 µs / 50 µs) |
| `power` | Signal power and noise-floor estimation |
| `tone` | Goertzel single-frequency detection |
| `window` | Window functions for spectrum analysis |
| `convert` | Raw interleaved bytes → complex samples |

`IqSample` is `num_complex::Complex<f32>` throughout — one IQ sample is one point on the
complex plane, which is the idea the whole talk is built around.
