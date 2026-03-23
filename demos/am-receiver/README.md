# am-receiver

Demodulates AM radio and plays it through your speakers. AM demodulation is even simpler than FM — just take the magnitude of each IQ sample.

## Run it

```bash
# Start rtl_tcp in another terminal first
rtl_tcp

# Listen to an AM station (replace with a local frequency)
cargo run -p am-receiver -- --frequency 1310000
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--source` | `tcp:127.0.0.1:1234` | SDR source |
| `--frequency` | `7850000` (7.85 MHz) | Station frequency in Hz |
| `--sample-rate` | `1024000` | Sample rate in Hz |
| `--audio-rate` | `48000` | Audio output sample rate in Hz |
| `--gain` | auto | Tuner gain in dB |

## How it works

AM radio encodes audio in the **amplitude** of the signal. The demodulation is:

```rust
input.iter().map(|s| s.norm()).collect()
```

Just take the magnitude of each IQ sample: `sqrt(I² + Q²)`. The distance from the origin *is* the audio.

Compare this to FM, where the audio is the *speed of rotation*. Same IQ data, different interpretation.

## Note on frequencies

The RTL-SDR's tuner (R828D) can receive from about 24 MHz to 1.7 GHz. Standard AM broadcast (530-1700 kHz) is below this range and requires direct sampling mode, which has driver compatibility issues. Aviation AM (118-137 MHz) works out of the box.
