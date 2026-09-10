# chu-decoder

Decodes the CHU shortwave time signal — Canada's radio broadcast of the national atomic
clock — into a displayed date and time.

> ## ⚠️ CHU went off the air permanently on 22 June 2026
>
> **This decoder can no longer receive anything live.** There is no signal to tune. The
> code is kept because the decoding problem is a good one and the station's format is
> worth reading, but if you run this against a dongle you will get silence, and that is
> not a bug in the program.
>
> It still runs against a recording: `--source file:PATH`. If you have a CHU capture from
> before the shutdown, it will decode.

## What CHU transmitted

CHU broadcast from Ottawa on 3330, 7850, and 14670 kHz. Between the audible seconds ticks
it sent a digital time code:

- **Bell 103-style FSK at 300 baud**, in bursts during seconds 31–39 of each minute.
- Each burst is **10 bytes** (8N2 framing, so 11 bits per byte): a 5-byte data block
  followed by a 5-byte redundancy block — identical for Format A, bit-complemented for
  Format B.
- **Format A** (seconds 32–39) carries day-of-year, hour, minute and second, all BCD,
  nibble-swapped.
- **Format B** (second 31) carries year, DUT1, TAI−UTC offset, and a DST code.

That redundancy block is the interesting part: it's how you decode a noisy shortwave
signal without a checksum, by demanding two independent copies agree.

## Options

| Flag | Default | Description |
|---|---|---|
| `--source`, `-s` | `tcp:127.0.0.1:1234` | SDR source: `file:PATH`, `tcp:HOST:PORT`, or `usb[:INDEX]` |
| `--frequency`, `-f` | `7850000` | CHU frequency in Hz |
| `--sample-rate` | `1024000` | Sample rate in Hz |
| `--gain`, `-g` | auto | Tuner gain in dB |
| `--direct-sampling` | `2` | Direct sampling branch for HF below ~24 MHz |
| `--no-setup` | off | Skip SDR setup commands (when `rtl_tcp` is already configured) |
| `--debug` | off | Show tone power levels |

## How it works

AM demodulate → low-pass filter → [Goertzel](https://en.wikipedia.org/wiki/Goertzel_algorithm)
tone detection for the two FSK tones → bit slicing → frame decode.

Goertzel rather than a full FFT because there are only two frequencies worth measuring,
and Goertzel measures exactly the bins you ask for at a fraction of the cost.

## Shortwave is hard indoors

Even before the shutdown, HF reception at a desk is usually noise-limited rather than
antenna-limited: the interference floor in a modern building swamps the signal, and a
bigger antenna indoors buys almost nothing. Going outside is what helps.
