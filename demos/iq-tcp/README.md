# iq-tcp

Streams IQ from a remote `rtl_tcp` to stdout **in exactly the format `rtl_sdr` writes**,
so the single-file receivers don't have to change at all.

```bash
iq-tcp --host radio.local --frequency 97.7 | fm-single
iq-tcp --host radio.local --frequency 119.9 --gain 40 | am-single
```

## Why this exists

The stage demos are `rtl_sdr … - | fm-single`. If the room has no signal, the dongle has
to move somewhere that does — a Raspberry Pi by a window running `rtl_tcp` — and the
laptop has to read it over the network instead.

[`fm-single`](../fm-single/) and [`am-single`](../am-single/) read raw bytes on stdin and
know nothing about where those bytes came from. So the only missing piece is something
that speaks `rtl_tcp` and writes those same bytes. This is it.

The file on the slide is unchanged. The only thing that moved is which process produces
the bytes.

## Options

```
iq-tcp --host HOST [--port 1234] --frequency MHZ
       [--sample-rate 960000] [--gain DB|auto]
       [--prebuffer-ms 500] [--stats] [--probe SECONDS]
```

| Flag | Default | Description |
|---|---|---|
| `--host`, `-H` | *required* | Remote host running `rtl_tcp` |
| `--port`, `-p` | `1234` | Remote port |
| `--frequency`, `-f` | *required* | Centre frequency in MHz |
| `--sample-rate`, `-s` | `960000` | Sample rate in Hz |
| `--gain`, `-g` | auto | Tuner gain in dB, or `auto` |
| `--prebuffer-ms` | `500` | Buffer before the first byte reaches the receiver |
| `--stats` | off | Report throughput while streaming |
| `--probe SECONDS` | — | Measure the link instead of streaming |

**`--probe` before the session, not during it.** It reports delivered rate against the
rate the receiver needs and exits non-zero if the link can't keep up — which is the
question you want answered before you're standing in front of people.

## The rtl_tcp protocol, in full

It's small enough to write out:

- On connect the server sends **12 bytes**: the magic `RTL0`, then the tuner type and the
  number of gain settings, both big-endian `u32`.
- The client sends **5-byte commands**: one command byte, then a big-endian `u32`
  argument.
- Everything the server sends after the header is IQ: unsigned 8-bit samples, interleaved
  I, Q, I, Q — byte for byte what `rtl_sdr -` writes.

That last line is the whole reason this is a ~300-line program and not a rewrite of the
receiver.
