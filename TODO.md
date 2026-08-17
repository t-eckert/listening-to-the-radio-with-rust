# TODO — Lock-week actions from the 2026-08-15 assumptions audit

Source: devil's-advocate review of every assumption in the deck, production notes, and
both repos. The talk's structure survived the audit; the safety nets did not. Items are
ordered roughly by risk. Each item says how to verify it — a diff is not evidence.

**Constraint for agents working this list:** Thomas hand-writes `skyward` code himself
(it's a learning project). For skyward items, provide review, test fixtures, and
diagnosis — ask before writing implementation code there. Items in *this* repo
(`listening-to-the-radio-with-rust`) are fair game to implement directly.

---

## 1. Cold-open fallback is not real — rebuild it

The stated mitigation ("pre-recorded IQ file one keystroke away") fails on three counts:
`demos/fm.iq` is 9.8 seconds long (18 MB at 960 kS/s × 2 bytes — the open needs 60–90 s),
it was captured in Ottawa (capture task defaults to 106.1, not 97.7), and nothing plays it
(`fm-single` reads stdin; no task exists).

- [x] Added `task fm-file` and `task am-file`. **Not** via the `while true; do cat` sketch
      above — that was tried first and measured at **61x realtime** (885 MB of WAV, 4,612
      audio-seconds, in 75 s of wall clock). `rtl_sdr` paces a live stream because the
      dongle really does produce samples in real time; a file has no clock, so the receiver
      is flooded and the audio queue grows without bound. It sounds terrible through a PA.
- [x] Added `demos/iq-play` — a dependency-free binary that loops a capture to stdout at the
      rate it was recorded (1,920,000 B/s = 960 kS/s of 8-bit I and Q, overridable). Both
      file tasks are now `bin/iq-play FILE | bin/<receiver>`.
- [ ] Document in `outline.md` that a fresh 2–3 min capture of 97.7 must be made at the
      venue during the break (`task capture FREQ=97.7 SECS=150 OUT=chom.iq`), and that the
      interim fallback is a longer home capture (see item 2).
- [x] **Verified (pacing):** 38,400,000 bytes delivered in 20.29 s = 1,892,904 B/s, a
      **0.986x** realtime factor, and 20.0 audio-seconds out of the 10.0 s `fm.iq`, so the
      loop works. Measured with `iq-play | head -c | wc -c`, no audio device in the path.
- [x] **Verified (failure modes):** missing file, empty file, and no-args all exit non-zero
      with a readable message; `task am-file` refuses with "record one with `task capture`"
      because `atc.iq` does not exist yet.
- [ ] **Still to verify by ear:** play `task fm-file` end to end and hear >60 s of continuous
      audio. Not done — pacing is measured, but nobody has listened to it.

## 2. No AM/ATC recording exists — record one

Risk Mitigation claims "pre-record IQ samples for all demos"; only FM exists. ATC is the
demo most likely to sit silent live, so it needs the recording most.

- [ ] (Thomas, at the desk) Record 2–3 min of ATC with actual transmissions on it:
      `task capture FREQ=<active Ottawa freq> SECS=180 OUT=atc.iq`. Ottawa tower is fine
      for a backup clip.
- [ ] Re-capture a longer FM clip too (current fm.iq is 10 s): `SECS=180`.
- [ ] **Verify:** play both through `task am-file` / `task fm-file` and hear voice/music.

## 3. "Why Rust" slide quotes the wrong receiver's numbers — fix the slide

The slide says "2.4 million samples per second, ~400 ns each." That is the ADS-B/Pi rate;
the FM receiver the audience just read runs at 960 kS/s (~1 µs/sample), stated three
slides earlier. Also, skyward's own `runs/baseline.json` shows the naive pipeline at
217–361× realtime, which undercuts "a GC pause makes it stop."

- [x] Corrected the rate on the "Why Rust" slide: now "960 thousand samples per
      second, ~1 µs each." (Corrected the one falsehood — the 2.4 MHz/400 ns rate —
      rather than a full rewrite; the existing bullets, "No C dependencies," and
      zero-cost-iterators claims were already accurate. The static-binary/213-tests
      enrichment is still available if you want to add it.)
- [x] Updated the speaker note's "2.4 million" quote to "960 thousand… about a
      microsecond each," plus a reminder that 2.4 MHz/400 ns is the ADS-B rate.
- [x] **Verify:** rendered slide 27 shows the corrected numbers; 960 kS/s traces to
      the FM/AM `RatePipeline`, and 1 µs = 1/960 kHz.

## 4. Cold-open script contradicts the deck — fix the wording

Script says "about forty lines of Rust"; "The Whole Radio" slide then shows a 327-line
file called "the entire receiver."

- [x] Changed the cold-open speaker note to "about forty lines of signal processing."
- [x] **Verify:** re-read the cold-open note (slide 1) and the FileMap slide together —
      the file is 327 lines total but Step 2 demodulate is 39 lines, so "forty lines of
      signal processing" no longer contradicts "the entire receiver."

## 5. Stage tasks compile on stage — alias the real binaries

`task fm-single` / `task am-single` invoke `cargo run --release` (dependency check at
best, rebuild if dirty, during the cold open). `task alias` links only the retired
multi-file receivers, not the two binaries the talk actually uses.

- [x] Added `fm-single`, `am-single`, and `iq-play` symlinks to the `alias` task.
- [x] All four stage/file tasks now invoke `bin/…` directly. Each carries a `preconditions`
      block naming the missing binary and saying to run `task alias` **before** the session
      rather than on stage — deliberately no `deps: [alias]`, since that would drag `build`
      back into the stage path and reintroduce the problem.
- [x] **Verified (partial):** no `cargo run` remains in `fm-single`, `am-single`, `fm-file`,
      or `am-file`; `task alias` links all three new binaries; `cargo build --release`
      finishes clean.
- [ ] **Still to verify:** `task fm-single` with the network disabled and the dongle
      attached, confirming zero compilation output before audio. Needs the hardware.

## 6. The Pi has never run skyward — prove the whole remote path (skyward repo; Thomas-led)

No evidence skyward has ever run on a Raspberry Pi: no aarch64 cross-build has been
performed (no target in `rust-toolchain.toml`; the guide's "non-event" line is a claim,
not a log), no systemd units exist, and the USB source is unimplemented — live operation
needs `rtl_tcp` as a separate process that nothing supervises. This is the closing
payoff's foundation and potentially days of yak-shaving.

- [ ] Cross-build (or build on the Pi) and run the full stack — `rtl_tcp` + `skyward run`
      — on the actual Pi with the real dongle and 7 cm antenna.
- [ ] Write two systemd units (`rtl_tcp.service`, `skyward.service`) with
      `Restart=always`, and enable them; the stack must survive a process kill and a
      power cycle unattended.
- [ ] Commit the uncommitted work: the entire `client/` UI, `web.rs`, `build.rs`, and
      modified files are working-tree-only. Six commits total, no CI.
- [ ] **Verify:** Pi runs unattended for several hours; `curl http://<pi>/healthz` and
      `/api/v1/aircraft` from another machine return live data; `kill` the rtl_tcp
      process and watch it recover.

## 7. Rehearse the local-replay ADS-B fallback (skyward repo; mostly Thomas)

The fallback path is real code (`--source file:` with `Pace::Realtime` and `--loop-file`;
`porch.cu8` holds 15 aircraft at 450 msg/min — likely richer than live Montreal will be).
But fixtures are gitignored and local-only (the `fixtures fetch` subcommand referenced in
`.gitignore` does not exist), and the switch has never been rehearsed.

- [ ] Ensure `porch.cu8` (864 MB) is deliberately present on the presentation laptop and
      that a laptop-local `skyward run --source file:fixtures/raw/porch.cu8 --loop-file`
      serves the map in a browser.
- [ ] Rehearse the live→replay switch once; write the trigger criterion into `outline.md`
      (e.g. "if the Pi is unreachable at 15:50, replay is primary — say so on stage,
      don't debug").
- [ ] **Verify:** browser at `localhost` shows a populated, updating map from the file
      source with the network off.

## 8. Venue unknowns — confirm this week (Thomas; human coordination, not code)

- [ ] Confirm the "7th floor by a window" room actually exists at the venue: access,
      power, permission, and network. The slide hardcodes it.
- [ ] Confirm the hardline Tina offered, and test laptop→Pi reachability on venue
      infrastructure — conference WiFi commonly has client isolation that silently
      blocks peer-to-peer even when both devices are online.
- [ ] Apply the placement lesson on site: antenna on the tripod ~40 cm back from the
      glass, never suction-cupped (low-E glass detunes a resonant 1090 MHz dipole;
      measured swing at home was 0.5 → 261 msg/min).
- [ ] Confirm with AV during the break that laptop audio can be live *before* the emcee
      introduction (the cold open depends on it). Add to the setup checklist in
      `outline.md`.

## 9. Timing has never been measured — do one full timed run-through (Thomas)

The 37.5-minute table is additive guesswork; cold opens and three live demos are exactly
what runs long.

- [ ] One complete timed rehearsal against the presenter view, demos included (file
      sources are fine). Record per-section times next to the table in `outline.md`.
- [ ] **Verify:** actual total ≤ 38:00, or apply the pre-planned cuts and re-time.

## 10. The Discord channel doesn't exist yet

The closing script promises "a Discord channel for this talk"; no URL exists anywhere.

- [ ] (Thomas) Create the channel/server, or cut the line from the closing note.
- [ ] If created: put the invite on the Getting Started slide, not just in the spoken
      close.

## 11. Steal skyward's headroom number for the "Four Stages" slide

Zero-cost upgrade found during the audit: `golden.toml` records that an integrating
detector finds **2,403** valid messages where the naive baseline finds **517** on the
same capture. Replace the abstract "lands beside the old one and gets scored" sentence
with the concrete stat — "the naive pipeline works, and four-fifths of the signal is
still on the table" — which turns the closing invitation into a hook for this audience.

- [x] Updated the ADS-B "Four Stages" slide + speaker note with the concrete stat:
      baseline 517 valid messages vs 2,403 with a smarter detector, "four-fifths of
      the signal is still on the table."
- [x] **Verify:** numbers match `golden.toml` `[headroom]` — `baseline_crc_ok = 517`,
      `better_detector_crc_ok = 2403` (517/2403 ≈ 0.22, so ~four-fifths on the table).
      Confirmed rendered on slide 44.

## 12. Minor consistency fixes

- [x] Fixed the `outline.md` cadence sentence: dropped the dead "physical object passed
      in front of the room" beat, now "sound, animation, a map filling in, and the
      hardware in the audience's hands at the close." Left a note offering the
      pass-the-dongle-during-RTL-SDR option if you want a mid-talk physical beat back.
- [x] Regenerated `radio-talk.pdf` via `npm run export` (400 KB → 432 KB, 46 pages,
      confirmed via `mdls`). Includes the two new ADS-B bit-encoding slides and all the
      edits above.
