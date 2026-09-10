# iq-play

Replays a recorded `.iq` file to stdout **at the rate it was captured**, looping forever.

```bash
iq-play fm.iq | fm-single
iq-play atc.iq | am-single
```

## Why this exists

`rtl_sdr` paces a live stream for free, because the dongle genuinely produces samples in
real time. A file has no such clock. So this:

```bash
cat fm.iq | fm-single        # don't
```

hands the receiver samples as fast as the pipe will carry them — measured at **61×
realtime** — which floods the audio queue and sounds like nothing you want coming out of
a PA system.

`iq-play` puts the clock back, and loops the file so a short capture can cover a long
segment.

## Usage

```
iq-play <file.iq> [bytes-per-second]
```

| Argument | Default | Description |
|---|---|---|
| `<file.iq>` | — | Raw 8-bit interleaved IQ, as written by `rtl_sdr` |
| `[bytes-per-second]` | `1920000` | 960 kS/s of 8-bit I and Q — the rate captures are recorded at |

If you captured at a different sample rate, pass twice that rate as the second argument.
