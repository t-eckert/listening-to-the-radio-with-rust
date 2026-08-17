#!/usr/bin/env python3
"""Report level and clipping for a recorded IQ file.

    ./capture-check.py fm.iq

A capture that clips is a distorted fallback, and nothing about the file size
says so — it looks exactly like a good one. Measured 2026-08-16: the 2 m whip on
a strong broadcast station clipped 10.6% of samples at gain 40 and 0% at 30.
Run this after every capture, before trusting it as a demo fallback.
"""

import sys

SDR_RATE = 960_000  # bytes per second is twice this: 8-bit I and Q


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: capture-check.py <file.iq>", file=sys.stderr)
        return 2

    path = sys.argv[1]
    try:
        with open(path, "rb") as f:
            data = f.read()
    except OSError as e:
        print(f"capture-check: cannot read {path}: {e}", file=sys.stderr)
        return 1

    if not data:
        print(f"capture-check: {path} is empty", file=sys.stderr)
        return 1

    # Skip the first tenth on long captures: the tuner settles and the AGC
    # moves for the first fraction of a second, which is not representative.
    body = data[len(data) // 10 :] if len(data) > 2_000_000 else data

    n = len(body)
    mean = sum(body) / n
    stdev = (sum((x - mean) ** 2 for x in body) / n) ** 0.5
    clipped = sum(1 for x in body if x <= 1 or x >= 254) / n * 100
    peak = max(abs(x - 127.5) for x in body)
    seconds = len(data) / (SDR_RATE * 2)

    print(
        f"{path}: {seconds:.1f} s, stdev {stdev:.1f}, "
        f"peak {peak:.1f}/127.5, clipped {clipped:.3f}%"
    )

    if clipped > 0.1:
        print("CLIPPING — lower GAIN and re-record")
        return 1
    if peak < 40:
        print("too quiet — raise GAIN and re-record")
        return 1
    print("level looks good")
    return 0


if __name__ == "__main__":
    sys.exit(main())
