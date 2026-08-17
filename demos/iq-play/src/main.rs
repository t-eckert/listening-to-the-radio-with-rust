//! Plays a recorded IQ file to stdout at the rate it was captured, looping forever.
//!
//!     iq-play fm.iq | fm-single
//!     iq-play atc.iq | am-single
//!
//! Why this exists: `rtl_sdr` paces a live stream for free, because the dongle
//! genuinely produces samples in real time. A file has no such clock, so
//! `cat file | fm-single` hands the receiver samples as fast as the pipe will
//! carry them — measured at 61x realtime here, which floods the audio queue and
//! sounds like nothing you want coming out of a PA system. This puts the clock
//! back, and loops the file so a short capture can cover a long cold open.

use std::env;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::process;
use std::thread;
use std::time::{Duration, Instant};

/// 960 kS/s of 8-bit I and Q — the rate `task capture` records at.
const DEFAULT_RATE: usize = 1_920_000;

/// A fiftieth of a second at the default rate. Small enough that the receiver
/// never starves, large enough that we are not syscalling in a tight loop.
const CHUNK: usize = 38_400;

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = match args.get(1) {
        Some(p) if !p.starts_with('-') => p,
        _ => {
            eprintln!("usage: iq-play <file.iq> [bytes-per-second]");
            eprintln!("       default rate {DEFAULT_RATE} B/s (960 kS/s, 8-bit I and Q)");
            process::exit(2);
        }
    };

    let rate: usize = match args.get(2) {
        Some(r) => match r.parse() {
            Ok(n) if n > 0 => n,
            _ => {
                eprintln!("iq-play: rate must be a positive number of bytes per second");
                process::exit(2);
            }
        },
        None => DEFAULT_RATE,
    };

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("iq-play: cannot open {path}: {e}");
            process::exit(1);
        }
    };

    // An empty file would loop forever without ever writing a byte, which looks
    // exactly like a hung demo. Fail loudly instead.
    match file.metadata() {
        Ok(m) if m.len() == 0 => {
            eprintln!("iq-play: {path} is empty");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("iq-play: cannot stat {path}: {e}");
            process::exit(1);
        }
        _ => {}
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut buf = vec![0u8; CHUNK];

    let start = Instant::now();
    let mut sent: u64 = 0;

    loop {
        let n = match file.read(&mut buf) {
            Ok(0) => {
                // End of the capture: rewind and keep going.
                if let Err(e) = file.seek(SeekFrom::Start(0)) {
                    eprintln!("iq-play: cannot rewind {path}: {e}");
                    process::exit(1);
                }
                continue;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("iq-play: read error on {path}: {e}");
                process::exit(1);
            }
        };

        // A write error here is the receiver having exited — Ctrl-C on the
        // pipeline, most often. That is a normal end, not a failure.
        if out.write_all(&buf[..n]).is_err() {
            return;
        }

        sent += n as u64;

        // Sleep until this many bytes *should* have been delivered. Deriving the
        // deadline from the running total rather than per-chunk means small
        // sleep overruns do not accumulate into drift.
        let due = Duration::from_secs_f64(sent as f64 / rate as f64);
        if let Some(remaining) = due.checked_sub(start.elapsed()) {
            thread::sleep(remaining);
        }
    }
}
