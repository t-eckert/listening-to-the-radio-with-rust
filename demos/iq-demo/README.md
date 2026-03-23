# iq-demo

An interactive visualization showing the relationship between a radio signal in the time domain (a sine wave) and its representation as a point rotating on the complex plane (IQ samples).

## Run it

```bash
cargo run -p iq-demo
```

### Controls

- **Left/Right arrows** — decrease/increase frequency
- **Up/Down arrows** — increase/decrease amplitude
- **q** — quit

## What you'll see

- **Left panel**: an oscilloscope-style view of the signal over time
- **Right panel**: a point rotating around the origin on the complex plane, with a line from the center showing the current IQ value

The two views are synchronized:

- **Faster rotation** on the right = **higher frequency** on the left
- **Larger circle** on the right = **greater amplitude** on the left

This is the key insight for SDR: the dongle gives you IQ samples (points on the complex plane), and all demodulation is about measuring either the rotation speed (frequency) or the distance from the origin (amplitude).

## No hardware needed

This demo is purely visual — no RTL-SDR dongle required.
