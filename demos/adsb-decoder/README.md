# adsb-decoder

Decodes ADS-B messages from aircraft transponders at 1090 MHz. Every commercial aircraft broadcasts its position, altitude, speed, and callsign — unencrypted, twice per second.

## Run it

```bash
# Start rtl_tcp in another terminal first
rtl_tcp

# Decode aircraft signals (uses a short ~7cm antenna for best results)
cargo run -p adsb-decoder
```

Aircraft will start appearing as their messages are decoded. The decoder stores results in `adsb.db` (SQLite), which the [flight-tracker](../flight-tracker/) TUI can display on a map.

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--source` | `tcp:127.0.0.1:1234` | SDR source |
| `--sample-rate` | `2400000` | Sample rate in Hz (2.4 MS/s recommended) |
| `-g, --gain` | auto | Tuner gain in dB |

## How it works

ADS-B is a **magnitude-based** signal — the same `s.norm()` operation as AM, but instead of audio, the on/off pattern encodes digital data.

The pipeline:
1. **Tune to 1090 MHz** with a short antenna (~7cm, quarter wavelength)
2. **Detect preambles** — ADS-B messages start with a specific pulse pattern
3. **Demodulate** — convert magnitude samples to bits using pulse position
4. **CRC check** — verify message integrity
5. **Decode** — parse the 112-bit message into callsign, position, altitude, speed

## Antenna

ADS-B works best with a short antenna. The ideal length is about 7 cm — a quarter wavelength at 1090 MHz. The long FM dipole will perform poorly at this frequency because of destructive interference (eddy currents) along its length.
