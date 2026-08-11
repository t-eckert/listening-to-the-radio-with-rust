# fm-receiver

Demodulates FM radio and plays it through your speakers. This is the simplest "useful" SDR demo — about 50 lines of signal processing code.

## Run it

```bash
# Start rtl_tcp in another terminal first
rtl_tcp

# Listen to an FM station (replace with a local frequency in MHz)
cargo run -p fm-receiver -- --frequency 100.3
```

You should hear a local FM station through your laptop speakers.

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--source` | `tcp:127.0.0.1:1234` | SDR source |
| `--frequency` | `100.3` | Station frequency in MHz |
| `--sample-rate` | `960000` | Sample rate in Hz. Must be a whole multiple of `--audio-rate` |
| `--audio-rate` | `48000` | Audio output sample rate in Hz |

### Why 960 kHz

The receiver decimates in two steps, and each step keeps every Nth sample — so
N has to be a whole number. 960000 → 240000 → 48000 divides evenly at both
steps. The old default of 2048000 did not: 2048000/48000 is 42.667, and
rounding that off produced audio at 51200 Hz labelled as 48000 Hz, which played
about 6.7% slow and a semitone flat. Rates that cannot divide evenly are now
rejected with an error rather than silently rounded. 1920000 and 2400000 also
work.

## How it works

FM radio encodes audio in the **rate of phase change** of the signal. The demodulation is one key operation:

```rust
let product = sample * self.prev.conj();
let phase = product.im.atan2(product.re);
```

Multiply each IQ sample by the conjugate of the previous sample, then take the angle. The result is the instantaneous frequency — which is the audio signal.

In IQ terms: the speed at which the point rotates around the origin *is* the audio.

The full pipeline: IQ samples → FM demodulation → low-pass filter → decimation → audio output.
