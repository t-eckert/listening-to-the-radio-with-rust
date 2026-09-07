//! A stand-in for `rtl_tcp` that serves a recorded IQ file instead of a dongle.
//!
//!     rtl-tcp-fake fm.iq                 # listens on 127.0.0.1:1234
//!     rtl-tcp-fake fm.iq --bind 0.0.0.0:1234
//!
//! Why this exists: the remote-receiver path has two halves, and only one of
//! them lives on the laptop. Without a Pi and a dongle in the room there is no
//! way to rehearse `iq-tcp | fm-single`, or to prove that it produces the same
//! audio the local path does. This serves the same greeting, accepts the same
//! commands, and paces the same bytes, so the client half can be exercised for
//! real against a file whose sound is already known.
//!
//! It is a test fixture and a rehearsal aid. It is not a radio, and nothing on
//! stage should ever point at it — if you can run this, you can run
//! `task fm-file`, which is one process shorter.

use std::env;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream};
use std::process;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_BIND: &str = "127.0.0.1:1234";

/// 960 kS/s of 8-bit I and Q — the rate `task capture` records at and the rate
/// the single-file receivers are compiled for.
const DEFAULT_RATE: usize = 1_920_000;

const CHUNK: usize = 38_400; // a fiftieth of a second at the default rate

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mut path: Option<String> = None;
    let mut bind = DEFAULT_BIND.to_string();
    let mut rate = DEFAULT_RATE;
    let mut once = false;

    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--bind" => {
                i += 1;
                match argv.get(i) {
                    Some(v) => bind = v.clone(),
                    None => fail("--bind needs an ADDR:PORT"),
                }
            }
            "--rate" => {
                i += 1;
                match argv.get(i).map(|v| v.parse::<usize>()) {
                    Some(Ok(n)) if n > 0 => rate = n,
                    _ => fail("--rate needs a positive number of bytes per second"),
                }
            }
            // Serve one client and exit, so a test can wait on the process.
            "--once" => once = true,
            other if !other.starts_with('-') && path.is_none() => path = Some(other.to_string()),
            other => fail(&format!("unknown argument {other:?}")),
        }
        i += 1;
    }

    let Some(path) = path else {
        fail("usage: rtl-tcp-fake <file.iq> [--bind ADDR:PORT] [--rate BYTES_PER_SEC] [--once]");
    };

    // Open it now rather than per-connection: a typo in the filename should
    // fail before anything believes this server is up.
    match File::open(&path) {
        Ok(f) => match f.metadata() {
            Ok(m) if m.len() == 0 => fail(&format!("{path} is empty")),
            Err(e) => fail(&format!("cannot stat {path}: {e}")),
            _ => {}
        },
        Err(e) => fail(&format!("cannot open {path}: {e}")),
    }

    let listener = match TcpListener::bind(&bind) {
        Ok(l) => l,
        Err(e) => fail(&format!("cannot bind {bind}: {e}")),
    };
    eprintln!("rtl-tcp-fake: serving {path} on {bind} at {rate} B/s");

    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let path = path.clone();
                let handle = thread::spawn(move || {
                    if let Err(e) = serve(stream, &path, rate) {
                        eprintln!("rtl-tcp-fake: client ended: {e}");
                    }
                });
                if once {
                    let _ = handle.join();
                    return;
                }
            }
            Err(e) => eprintln!("rtl-tcp-fake: accept failed: {e}"),
        }
    }
}

fn fail(msg: &str) -> ! {
    eprintln!("rtl-tcp-fake: {msg}");
    process::exit(2);
}

fn serve(stream: TcpStream, path: &str, rate: usize) -> io::Result<()> {
    stream.set_nodelay(true)?;

    // The same 12-byte greeting rtl_tcp sends: magic, tuner type, gain count.
    // Tuner 5 is the R820T, which is what the RTL-SDR Blog V3 carries.
    let mut header = [0u8; 12];
    header[0..4].copy_from_slice(b"RTL0");
    header[4..8].copy_from_slice(&5u32.to_be_bytes());
    header[8..12].copy_from_slice(&29u32.to_be_bytes());
    (&stream).write_all(&header)?;

    // Commands arrive on the same socket while we are writing samples down it,
    // so they need their own thread. We only log them — there is no tuner to
    // apply them to — but logging is exactly what a test wants to assert on.
    let mut command_reader = stream.try_clone()?;
    thread::spawn(move || {
        let mut cmd = [0u8; 5];
        while command_reader.read_exact(&mut cmd).is_ok() {
            let value = u32::from_be_bytes([cmd[1], cmd[2], cmd[3], cmd[4]]);
            let name = match cmd[0] {
                0x01 => "set_freq",
                0x02 => "set_sample_rate",
                0x03 => "set_gain_mode",
                0x04 => "set_gain",
                0x09 => "set_direct_sampling",
                _ => "other",
            };
            eprintln!("rtl-tcp-fake: cmd {:#04x} {name} {value}", cmd[0]);
        }
    });

    let mut file = File::open(path)?;
    let mut out = stream;
    let mut buf = vec![0u8; CHUNK];
    let start = Instant::now();
    let mut sent: u64 = 0;

    loop {
        let n = match file.read(&mut buf)? {
            0 => {
                // End of the capture: rewind, the way a dongle never stops.
                file.seek(SeekFrom::Start(0))?;
                continue;
            }
            n => n,
        };
        out.write_all(&buf[..n])?;
        sent += n as u64;

        // Pace against the running total rather than per-chunk, so small sleep
        // overruns do not accumulate into drift.
        let due = Duration::from_secs_f64(sent as f64 / rate as f64);
        if let Some(remaining) = due.checked_sub(start.elapsed()) {
            thread::sleep(remaining);
        }
    }
}
