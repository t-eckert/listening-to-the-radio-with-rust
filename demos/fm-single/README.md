# fm-single

A complete FM broadcast receiver in **one file, with no project dependencies**. This is
the file that goes on screen during the talk: antenna to audio, top to bottom, in one
scroll.

`fm-receiver` is the same radio built properly, using the shared [`sdr`](../sdr/)
library. This one deliberately gives that up so nothing is hidden behind an import.

## Run it

It reads raw 8-bit IQ on stdin and plays audio on your default output, so `rtl_sdr`
does the tuning and this does the radio:

```bash
rtl_sdr -f 97700000 -s 960000 -g 30 - | fm-single

# add a path to also record what you hear
rtl_sdr -f 97700000 -s 960000 -g 30 - | fm-single recording.wav
```

Replace `97700000` with a station that's strong where you are — a published frequency
for your city is a starting guess, not a measurement. [`freq-scanner`](../freq-scanner/)
will tell you what actually reaches you.

No hardware? Feed it a recording instead, via [`iq-play`](../iq-play/):

```bash
iq-play fm.iq | fm-single
```

## The whole design, in three numbers

```
960 kHz  →  240 kHz  →  48 kHz
   what the dongle    after step 1    what the speakers want
      produces
```

Each rate divides evenly into the one above it (960/4 = 240, 240/5 = 48), so every
decimation step keeps a whole number of samples. Pick rates that don't divide evenly and
you get a slow drift between how fast audio is produced and how fast the sound card
consumes it — which you hear as continuous dropouts rather than an error.

## How it works

1. **Shift and narrow** — decimate 960 kHz down to 240 kHz, keeping the ~200 kHz the FM
   channel occupies.
2. **Demodulate** — multiply each IQ sample by the conjugate of the previous one and take
   the angle. The *rate the point rotates* around the origin is the audio.
3. **De-emphasise and decimate** — undo the treble boost the transmitter applied (75 µs
   in North America, 50 µs in Europe) and drop to 48 kHz.
4. **Play** — hand the samples to the sound card across an `Arc<AudioRing>`.

Read [`am-single`](../am-single/) next and diff the two files. Step 2 measures distance
from the origin instead of rotation, step 3 subtracts the slow-moving part instead of
keeping it, and that difference is the entire distinction between AM and FM.
