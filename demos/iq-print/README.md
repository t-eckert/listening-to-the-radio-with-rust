# iq-print

Prints raw IQ samples from the SDR to the terminal so you can see the actual data coming off the dongle.

## Run it

```bash
# Start rtl_tcp in another terminal first
rtl_tcp

# Print 10 IQ samples from a local FM station
cargo run -p iq-print -- --frequency 100.3
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--source` | `tcp:127.0.0.1:1234` | SDR source (tcp, usb, or file) |
| `--frequency` | `100.3` | Center frequency in MHz |
| `--sample-rate` | `2048000` | Sample rate in Hz |
| `-c, --count` | `10` | Number of samples to print |
| `-d, --delay` | `500` | Delay between prints in ms |

## What you'll see

Pairs of I and Q values — each pair is a point on the complex plane. These are the raw numbers that every other demo processes. At an FM station, you'll see the values changing rapidly as the signal modulates.

## How it works

The SDR dongle outputs interleaved bytes: I, Q, I, Q, ... Each byte is an 8-bit unsigned integer (0-255). We convert them to floating point values between -1.0 and 1.0, which gives us complex samples on the unit circle.
