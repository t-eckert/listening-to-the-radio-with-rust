# wave-demo

A terminal animation showing electromagnetic wave propagation from a transmitting antenna to a receiving antenna.

## Run it

```bash
cargo run -p wave-demo
```

Press `q` to quit.

## What you'll see

- A **transmitting antenna** on the left with an electron (dot) oscillating up and down
- A **sine wave** propagating across the screen from left to right
- A **receiving antenna** on the right whose electron begins oscillating once the wave arrives

This is the core physical concept behind all radio: accelerating charges in the transmitter create electromagnetic waves, and those waves push charges in the receiver. Everything else in SDR is about measuring and interpreting that movement.

## No hardware needed

This demo is purely visual — no RTL-SDR dongle required.
