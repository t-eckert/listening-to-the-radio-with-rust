# freq-scanner

Sweeps a frequency range, measures the power at each step, and shows you what's actually
receivable **where you're sitting** — with a live spectrum display.

```bash
# Start rtl_tcp in another terminal first
rtl_tcp

cargo run -p freq-scanner                    # sweeps the FM band, 88-108 MHz
cargo run -p freq-scanner -- --headless      # ranked list on stdout, no TUI
```

## Why you want this before anything else

A published frequency list for your city tells you what's *transmitted*. It does not tell
you what *arrives* at your desk, on your antenna, on the far side of your building. Those
are different questions, and the second one is the one that decides whether a demo works.

This was learned the hard way. A frequency measured at a venue one afternoon read as pure
noise from the stage of the same venue the next day — same building, a few metres apart.
Scanning from the actual spot found a usable station in about two minutes.

**Scan first. Swap hardware last.**

## Options

| Flag | Default | Description |
|---|---|---|
| `--source`, `-s` | `tcp:127.0.0.1:1234` | SDR source: `file:PATH`, `tcp:HOST:PORT`, or `usb[:INDEX]` |
| `--start` | `88.0` | Start frequency in MHz |
| `--end` | `108.0` | End frequency in MHz |
| `--step` | `100.0` | Step size in kHz |
| `--sample-rate` | `2048000` | Sample rate in Hz |
| `--dwell` | `50` | Dwell time per step in ms |
| `--threshold` | `6.0` | Detection threshold in dB above the noise floor |
| `--gain`, `-g` | auto | Tuner gain in dB |
| `--headless` | off | Log detections to stdout instead of drawing a TUI |

## A note on dwell time

The default 50 ms per step is a deliberate trade. Too short and you measure the tuner
settling rather than the band — the PLL needs time to lock after each retune, and a
sub-100 ms grab taken right after tuning is dominated by that transient. If a scan gives
you results that don't reproduce, raise `--dwell` before you doubt the antenna.
