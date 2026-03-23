# fm-receiver

Demodulates FM radio and plays it through your speakers. This is the simplest "useful" SDR demo — about 50 lines of signal processing code.

## Run it

```bash
# Start rtl_tcp in another terminal first
rtl_tcp

# Listen to an FM station (replace with a local frequency)
cargo run -p fm-receiver -- --frequency 100300000
```

You should hear a local FM station through your laptop speakers.

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--source` | `tcp:127.0.0.1:1234` | SDR source |
| `--frequency` | `100300000` (100.3 MHz) | Station frequency in Hz |
| `--sample-rate` | `2048000` | Sample rate in Hz |
| `--audio-rate` | `48000` | Audio output sample rate in Hz |

## How it works

FM radio encodes audio in the **rate of phase change** of the signal. The demodulation is one key operation:

```rust
let product = sample * self.prev.conj();
let phase = product.im.atan2(product.re);
```

Multiply each IQ sample by the conjugate of the previous sample, then take the angle. The result is the instantaneous frequency — which is the audio signal.

In IQ terms: the speed at which the point rotates around the origin *is* the audio.

The full pipeline: IQ samples → FM demodulation → low-pass filter → decimation → audio output.
