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
- [x] **Verified by ear (2026-08-17):** `task fm-file` plays clean, continuous FM. The new
      `fm.iq` is a 120 s capture of a strong local station (see item 2 for why not 97.7).

## 2. No AM/ATC recording exists — record one

Risk Mitigation claims "pre-record IQ samples for all demos"; only FM exists. ATC is the
demo most likely to sit silent live, so it needs the recording most.

### Capture session 2026-08-17 (V4 `FMAM01`, 2 m vertical against office window)

**FM — done, but the frequency assumption was wrong.** `fm.iq` is now a clean 120 s
capture of **101.7 (Ottawa, g20, 0% clipped)**, verified by ear. Key finding: **97.7 is
CHOM *Montreal*** — ~200 km away, so at the Ottawa desk it demods to fuzz. Home captures
MUST use a strong *local* station. Measured at the window: 97.9 and 101.7 are crystal
clear; CKCU 93.1 (campus, low power) is too hissy indoors even at g40. The deck's live
cold open still uses 97.7 — that gets captured fresh at the **Montreal venue**, where
97.7 *is* CHOM.

Gain also had to drop hard: a strong local station rails the ADC at g30 (37% clipped on
97.7-band, 26% on 101.7). Good levels were g10–g20 for commercial FM. The old Taskfile
comment (g40 → 10.6% clipped) understated it.

- [x] FM clip recorded and verified (101.7, g20, 120 s → `fm.iq`).
- [ ] **Interim only** — Thomas plans to record a better FM clip *outdoors* later. Overwrite
      `fm.iq` when that exists.
- [ ] Update the `task capture`/`fm-single` FM defaults away from 97.7 for home use, or add a
      `LOCAL` note, so nobody re-captures Montreal's frequency at the desk by accident.

**Aviation (ATC) — cannot be captured at this window.** ATIS (121.15) had no voice; an
airband power scan (118–137 MHz) showed 121.15 absent and everything else only 5–11 dB
over the noise floor (135.15 approach, a ~132.9 cluster). `atis.iq` / `tower.iq` /
`ground.iq` are on disk but demod to noise. A 2 m vertical indoors is too weak for
118–137 MHz here.

- [ ] Capture ATC **outdoors or at the venue**, not at the office window. Scan the airband
      on site (`rtl_power -f 118M:137M:25k`) and record the strongest active frequency.
- [ ] `task am-file` still defaults to `atc.iq`, which does not exist. Once a real ATC clip
      exists, name it `atc.iq` (or point the task default at whatever the primary clip is).

**MW broadcast — dropped.** The 2 m vertical is hopelessly short for the 530–1700 kHz
band; `mw.iq` demodulated to unintelligible mush and was deleted. Not a core demo.

## 3. "Why Rust" slide quotes the wrong receiver's numbers — fix the slide

The slide says "2.4 million samples per second, ~400 ns each." That is the ADS-B/Pi rate;
the FM receiver the audience just read runs at 960 kS/s (~1 µs/sample), stated three
slides earlier. Also, skyward's own `runs/baseline.json` shows the naive pipeline at
217–361× realtime, which undercuts "a GC pause makes it stop."

- [x] Corrected the rate on the "Why Rust" slide: now "960 thousand samples per
      second, ~1 µs each." (Corrected the one falsehood — the 2.4 MHz/400 ns rate —
      rather than a full rewrite; the existing bullets, "No C dependencies," and
      zero-cost-iterators claims were already accurate. The static-binary/213-tests
      enrichment is still available if you want to add it. **Correction 2026-08-24:
      it's 206 tests, not 213** — `cargo test --workspace` in skyward reports
      `passed: 206 failed: 0` across 11 binaries. 213 is the count of `#[test]`
      attributes in the tree, which is not the same thing. Use 206 if it goes on a
      slide. The static-binary claim is still unverified — see item 6.)
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

**Most of this item is stale as of 2026-08-24 — skyward has moved.** Re-checked against
the repo while correcting the deck's `rtl_tcp` wording (item 15). Corrections:

- ~~"the USB source is unimplemented"~~ — **it is implemented.** `be08fa8 source: the
  dongle over USB, and a link that comes back`. `UsbSource::open` binds librtlsdr
  directly; `SKYWARD_SOURCE=usb` is now the *recommended* deployment and `rtl_tcp` is the
  fallback. There is no second process to supervise.
- ~~"no systemd units exist"~~ — **they do.** `deploy/systemd/skyward.service` and
  `rtl_tcp.service`, from `f251a20`.
- ~~"the uncommitted work is working-tree-only"~~ — **committed.** Only an untracked
  `LICENSE` remains in the working tree.
- There is now `docs/RASPBERRY_PI.md` (a step-by-step deploy guide) and
  `docs/PI_AUDIT.md` (a 12-finding readiness audit, 9 fixed).

**What still stands, and it is the important part.** `docs/PI_AUDIT.md` states its own
boundary plainly: *"No step was run on a Raspberry Pi, and no RTL-SDR was attached."*
Everything above is code and documentation, not a Pi that has ever decoded an aircraft.
`rust-toolchain.toml` still pins only `1.94.1` with no aarch64 target, and the audit's
finding 3 says cross-compiling needs a C cross-toolchain and is not dependency-free.

- [ ] Build (on the Pi, or cross with the C toolchain the audit names) and run
      `skyward run` on the actual Pi with the real dongle and 7 cm antenna.
      **Use `--features usb`** — the direct-USB path is the one the docs recommend and
      the one the talk now describes on stage.
- [ ] Install and enable `deploy/systemd/skyward.service`; confirm it survives a process
      kill and a power cycle unattended. (`rtl_tcp.service` is only needed for the
      fallback shape.)
- [ ] **Verify:** Pi runs unattended for several hours; `curl http://<pi>/healthz` and
      `/api/v1/aircraft` from another machine return live data; `kill` the skyward
      process and watch systemd bring it back.
- [ ] **Verify the claim the talk now makes out loud:** that the Pi is doing the whole
      radio and serving answers. `curl http://<pi>/api/v1/aircraft` from the laptop is
      that proof.

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
      power, permission, and network. **The slide no longer hardcodes the floor** — it
      reads "upstairs, by a window" as of 2026-08-24, so the deck survives being given a
      different room. This is now an operational question only, not a slide dependency.
- [ ] Confirm the hardline Tina offered, and test laptop→Pi reachability on venue
      infrastructure — conference WiFi commonly has client isolation that silently
      blocks peer-to-peer even when both devices are online.
- [ ] Apply the placement lesson on site: antenna on the tripod ~40 cm back from the
      glass, never suction-cupped (low-E glass detunes a resonant 1090 MHz dipole;
      measured swing at home was 0.5 → 261 msg/min).
- [ ] Confirm with AV during the break that laptop audio can be live *before* the emcee
      introduction (the cold open depends on it). Add to the setup checklist in
      `outline.md`.

## 9. Timing measured at 32:00 — no pre-emptive cuts (Thomas) — DONE

- [x] **Measured 32:00 on 2026-08-19**, timed run-through with Ernest Kissiedu.
- [x] **Verified:** 32:00 is under the 38:00 bar, so the pre-planned cuts are **not**
      applied pre-emptively. They remain a live contingency behind the TIME CHECK cues.
- [ ] Per-section times were not captured, only the total. Take them at the next
      rehearsal if one happens before Montreal. **This is now the highest-value timing
      item:** the deck's speaker-note clock is rebased on a single uniform 0.84 scaling
      of this run (see item 13), and per-section times would replace that guess with
      measurements.

The estimate table in `outline.md` ran 7.5 minutes long; it has since been rebuilt from
the deck's calibrated stamps (item 13) and no longer disagrees. Note that 32:00 understates the
live figure: section 9's ATC listen could not be rehearsed (`atc.iq` does not exist), and
live delivery drifts longer than rehearsal.

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

## 13. Ernest's feedback: "more Rust cheerleading" — DONE

Timed run-through 2026-08-19. Feedback was overwhelmingly positive (cadence, presence,
makes a complex subject understandable). One substantive note: **say more about why Rust
and what makes it good for this**.

History: `c2614e6` added a "Why Rust" slide; it is present in `66cd6f9` and gone in
`3a26ba6` (the FM/AM pipeline rework, 2026-08-18 21:41). Its crate table survived as the
standalone "The Crates" slide; the argument did not. So the deck said "Rust" in the title,
the intro, and a crate list, and nowhere made a case.

**That removal was deliberate, not accidental** (corrected 2026-08-24 — an earlier note in
this file called it an accident, which was wrong). The 2026-08-18 decision was recorded at
the time with an explicit rationale: *"no point selling Rust at RustConf."* Ernest's
run-through the next day is the evidence against that reasoning — the room he stood in
wanted the case made. So this is a reversal of a judgement call, not the repair of a
mistake, and the rewrite reflects that: it argues from the constraint and from code the
audience has just read, rather than selling the language.

- [x] **Restored "Why Rust"** as slide 30, after "FM: The Whole Loop" — rewritten, not
      reverted. Argues from the `Arc<AudioRing>` hand-off at `fm-single/src/main.rs:213`
      (a real two-thread share in the file the audience just read) rather than three
      abstract bullets. 1:15.
- [x] **Added "ADS-B: What 112 Bits Actually Say"** as slide 45, between "Why Timing Is
      Everything" and "Four Stages". Double duty, as requested: the audience sees the
      `Message` enum from `skyward/crates/adsb-core/src/decode.rs` — so they see what an
      aircraft actually transmits — and the `Knots`/`TrackDeg`/`FeetPerMinute` newtypes
      make the Rust case without a bullet list. Closes on the real comment from
      `units.rs`: *"naming it wrong is the kind of error that survives all the way onto a
      conference slide."* 1:15.
- [x] **Typestate click on "ADS-B: Four Stages"** — `Candidate → RawFrame → Validated`,
      so "a wrong altitude is dangerous" is enforced by the type rather than by
      discipline. One `<v-click>`, no new slide. +0:15.
- [x] **Seasoning:** the "Three Things" note now promises three Rust stops (mirroring
      music / a voice / aircraft), and "The Crates" note gained the dependency-count
      argument.
- [x] **Verified (numbers):** `cargo tree -p fm-single` → 2 direct deps, 10 crates in the
      whole tree. Both slide claims trace to source: `decode.rs`, `units.rs`, `types.rs`,
      `fm-single/src/main.rs:213`.
- [x] **Verified (renders):** `npx slidev build` clean; `npx slidev export --format png
      --range 30,45,46` produced all three and they were inspected — code blocks fit, no
      overflow, click-step highlighting lands on the intended lines.
- [x] **Verified (deck integrity):** `@slidev/parser` reports **51 slides**; the exported
      `radio-talk.pdf` contains **51 pages**. Both new slides appear at 30 and 45.
- [x] **Re-stamped every speaker-note timestamp.** The `[dur · cum]` series is now a true
      running sum again (it was, before the inserts; verified programmatically). All six
      absolute TIME CHECK cues shifted to match.
- [x] **SUBMITTED 2026-08-24** as `Eckert_Thomas_Listening-to-the-Radio-with-Rust.pdf`
      via the RustConf slides form. 51 pages, 2,617,917 bytes, sha256 `3780f61f…`,
      735.12 × 414 pt (16:9). Page 1 rasterized and confirmed non-blank before sending
      (see item 14 for the silent blank-export failure mode). Contains the restored
      "Why Rust" slide, the ADS-B message-types slide, the typestate click, the About Me
      photo, the calibrated 35:05 note clock, and the corrected rtl_tcp wording.

**The deck is now free to change again.** The organizers consider the submitted file
final and ask for no last-minute changes, but breakout speakers present from their own
laptop (item 16), so the projected deck and the archived PDF are different artifacts.
Anything that lands between now and 9 September will be seen live and not in the
submission. Keep changes worth that gap — and if the Discord URL arrives, adding it to
"Getting Started" is exactly the kind of change that qualifies.

### Timing: the note clock is now calibrated, not estimated

- [x] **Rescaled the whole `[duration · cumulative]` series onto the measured run.** The
      old series summed to 38:05 on the run that measured **32:00**, so every slide that
      run covered was multiplied by **0.8403** (1920/2285). The 2:55 of material added
      since (Why Rust 1:15, message types 1:15, typestate +0:15, Rust thesis +0:10) has
      never been spoken, so it was left at face value. **32:00 + 2:55 = 34:55**, and the
      series lands exactly there.
- [x] Cumulatives rounded to 5 s and durations derived back out of them, so rounding
      cannot accumulate down the deck.
- [x] All eleven absolute TIME CHECK times remapped through the same curve by
      interpolation (thresholds fall between slides, so a flat multiply would have been
      wrong).
- [x] **Verified:** 51 stamps, series is an exact running sum, total 34:55, no slide
      rounded to a zero duration, and all five "leaving this slide" checkpoints equal
      their own slide's cumulative.
- [x] Rebuilt the `outline.md` timing table from the deck's real stamps rather than from
      estimates — and fixed its section boundaries, which had drifted out of sync with
      the deck after the no-antenna-swap rework (§11 "antenna length" is slide 10, in the
      physics section; the "receiver isn't in this room" payoff is slide 39, in ADS-B).
      All 51 slides are now accounted for.
- [ ] **The 0.8403 factor is a guess, not a measurement.** It assumes every section ran
      long by the same proportion, which is certainly false. Replace it with real
      per-section times at the next rehearsal — see item 9.

**34:55 against a 38:00 target and a 40:00 ceiling.** Treat it as a floor, not spare
time: section 9's ATC listen (~75 s) has never been rehearsed because `atc.iq` does not
exist, and live delivery drifts longer than rehearsal.

## 14. About Me photo — DONE

- [x] `deck/public/me.jpg` added from `~/Desktop/IMG_8241.jpeg`, and the PHOTO
      PLACEHOLDER div on the About Me slide replaced with
      `<img src="/me.jpg" alt="Thomas Eckert" class="about-photo" />`.
- [x] **Rotation baked in, EXIF neutralised.** The original carries EXIF
      `Orientation = 6` over landscape pixel data — it *displays* portrait only because
      viewers honour the tag. `sips` copies the tag without applying it, so a naive
      resize produced a file whose pixels were still landscape. The asset is now genuinely
      portrait (1200×1600) with the tag rewritten to `1`, so nothing downstream can
      double-rotate it. Original left untouched on the Desktop; it is the only
      full-resolution copy (4284×5712).
- [x] Added `object-position: 50% 30%` so `object-fit: cover` crops to the face rather
      than the frame centre.
- [x] **Verified:** slide 2 rendered to PNG and inspected — upright, face well placed in
      the right half, nothing cropped off the head.

**Watch out when exporting.** `npm run export` silently produced a **12 KB, 51-page,
entirely blank** PDF once during this session. It reported `✓ exported to
./radio-talk.pdf` exactly as it does on success. Re-running it produced the correct
2.5 MB file. **Always check the byte size after exporting** — a healthy export of this
deck is >2 MB — and ideally open page 1. A blank submission on Aug 25 would be
unrecoverable.

## 15. The Pi link is HTTP, not `rtl_tcp` — wording corrected

The deck spends a slide on `rtl_tcp` ("From Dongle to Your Code") and then, forty minutes
later, said this at the remote-receiver reveal:

> Remember that socket, forty minutes ago. **This is what it was for.**

That is wrong. `skyward` is a hosted application on the Pi: it reads the dongle directly
over USB and serves JSON and SSE. Raw IQ never crosses the network. The line implied the
audience was watching an `rtl_tcp` stream come down from upstairs.

- [x] Rewrote that beat as a deliberate **contrast** rather than a match: `rtl_tcp` *would*
      let you split dongle from decoder, "and that is not what I did" — 2.4 MS/s is ~4.8
      MB/s of raw IQ, which is not going across conference WiFi. The Pi runs the whole
      radio and sends answers. Costs ~10 s and is a better beat than the one it replaces.
- [x] Fixed the production aside on "From Dongle to Your Code", which told you to set up a
      callback the ADS-B section would then contradict.
- [x] Fixed the `RemoteReceiver.vue` header comment, which said the diagram "collects the
      earlier socket callback" — the wire it draws is HTTP. Added a note not to relabel it
      with a socket or a sample stream.
- [x] Added "The ADS-B link is HTTP, not `rtl_tcp`" to `outline.md`, with the 4.8 MB/s
      reasoning from `skyward/docs/RASPBERRY_PI.md`.
- [x] **The `rtl_tcp` slide itself is unchanged**, as requested. Its claim — that a socket
      means the dongle need not be on the same machine as your code — is true and is now
      the setup for a contrast instead of a false promise.
- [x] **Verified:** no remaining reference in `deck/slides.md` ties `rtl_tcp` or a socket
      to the Pi; the only `socket`/`tcp` mentions are on the transport slide itself and in
      the new explicitly-corrective note.

## 16. Speaker-packet findings (2026-08-24)

Read the RustConf speaker packet and the Aug 20 slides reminder. Answers the question of
what the Aug 25 PDF actually is, and turns up three items that matter more than the deck.

- [x] **The PDF is a backup, not the projected artifact.** *"We require all breakout
      session speakers to bring their own laptops... You will present the slides from your
      own laptop."* Keynotes/project updates/lightning talks use the AV machine; breakouts
      do not. So Slidev runs live and the click-steps survive — **the 127-page
      `--with-clicks` export is not needed.**
- [x] **Submission requirements checked against `radio-talk.pdf`:** 16:9 confirmed
      (735.12 × 414 pt = 1.7757 vs 1.7778, sub-pixel rounding); PDF accepted for
      breakouts; Inter, Fira Code and KaTeX all embedded and subsetted, so the
      "embed custom fonts" requirement is met.
- [ ] **The organizers recommend prerecording live demos.** *"Please don't rely on WiFi
      for your presentation... If you are planning a live demo, we recommend prerecording
      it."* That is aimed squarely at the laptop→Pi ADS-B payoff. **This makes item 7
      (rehearse the local-replay fallback) the highest-value open item** — the fallback is
      pre-blessed.
- [ ] **There is a scheduled radio test at the venue: Tuesday 8 September, 1–2 pm**, with
      Tina Krauss at the Registration Desk, Level 5 (attendance marked optional). Nearly
      every unchecked item in item 8 is answerable in that hour: confirm 97.7 and 119.9,
      scan the airband for a live ATC frequency to record `atc.iq` (item 2), and find out
      where the Pi can actually live.
- [x] **Item 10 is mostly retired.** The packet: *"As a speaker, you will receive a
      dedicated Discord channel"* and *"our team will share access information with you
      directly the week before the conference."* So the channel is provided, not
      requested, and it is safe to promise on stage — but the URL will not exist until
      early September. **Note a contradiction:** the Aug 20 email lists "Request dedicated
      Discord channel (optional)" as an Aug 25 action item while the packet says it is
      automatic. Worth one email to clarify.
- [ ] Tech prep for breakouts is **15 minutes prior to start time** at the AV table, which
      confirms this file's existing note. The 30-minute break plan is still the right one,
      but it is not something the packet promises.
- [x] Recorded and livestreamed: all sessions go to the Rust Foundation YouTube channel.

## 17. The deck does not run offline — fonts come from Google

`outline.md` claimed "Runs entirely offline once installed." **Measured false.**

- [x] `npx slidev build` output contains
      `fonts.googleapis.com/css2?family=Inter:wght@200;400;600&family=Fira+Code:...`.
      Only the KaTeX faces are bundled locally.
- [x] **Verified behaviourally** with Playwright against the built deck: online, two
      requests reach Google Fonts and `document.fonts` reports Inter loaded. With
      `fonts.googleapis.com` and `fonts.gstatic.com` blocked, Inter and Fira Code both
      report *not* loaded and the deck falls back to the system stacks.
- [x] **Impact measured, not assumed:** slide 1 and slide 30 (code) were screenshotted
      both ways. Near-identical, no reflow, code alignment preserved. The fallback is
      genuinely fine.
- [x] Corrected the false claim in `outline.md`.
- [ ] **After the Aug 25 submission**, vendor Inter and Fira Code into `deck/public` with
      local `@font-face` rules and set the Slidev font provider to none. Cheap, and it
      removes a network dependency the packet explicitly warns about. **Deliberately not
      done before the deadline** — a font change the day before submission risks a visual
      regression across 51 slides for a benefit measured at nearly zero.

## 12. Minor consistency fixes

- [x] Fixed the `outline.md` cadence sentence: dropped the dead "physical object passed
      in front of the room" beat, now "sound, animation, a map filling in, and the
      hardware in the audience's hands at the close." Left a note offering the
      pass-the-dongle-during-RTL-SDR option if you want a mid-talk physical beat back.
- [x] Regenerated `radio-talk.pdf` via `npm run export` (400 KB → 432 KB, 46 pages,
      confirmed via `mdls`). Includes the two new ADS-B bit-encoding slides and all the
      edits above.
