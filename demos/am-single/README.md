# am-single

A complete AM receiver in one file, with no project dependencies — the AM counterpart to
[`fm-single`](../fm-single/).

**Read it as a diff.** It is `fm-single` with two things changed:

- Step 2 measures **distance from the origin** instead of **rotation**.
- Step 3 **subtracts** the slow-moving part instead of **keeping** it.

That's the whole difference between AM and FM. The three sample rates are deliberately
identical so the two files stay diffable side by side.

## Run it

```bash
rtl_sdr -f 119900000 -s 960000 -g 40 - | am-single

# add a path to also record what you hear
rtl_sdr -f 119900000 -s 960000 -g 40 - | am-single recording.wav
```

Or replay a recording with [`iq-play`](../iq-play/):

```bash
iq-play atc.iq | am-single
```

## A note on gain

The example above uses gain 40 where `fm-single` uses 30, and that isn't arbitrary.
Aviation AM is weak and intermittent — the opposite problem to broadcast FM, where a
strong local station will rail the ADC if you leave the gain high. Don't copy a gain
figure from one to the other.

Aviation voice is also genuinely quiet most of the time. Silence usually means nobody is
talking, not that the receiver is broken.

## Channel widths

| | `fm-single` | `am-single` |
|---|---|---|
| Channel half-width | 100 kHz | 12.5 kHz (aviation channels are 25 kHz apart) |
| Audio bandwidth | 15 kHz (music) | 6 kHz (speech) |
