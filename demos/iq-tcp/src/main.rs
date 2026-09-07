//! Streams IQ from a remote `rtl_tcp` to stdout, in the same format `rtl_sdr`
//! writes, so the single-file receivers do not have to change at all.
//!
//!     iq-tcp --host radio.local --frequency 97.7 | fm-single
//!     iq-tcp --host radio.local --frequency 119.9 --gain 40 | am-single
//!
//! Why this exists: the stage demos are `rtl_sdr … - | fm-single`. If the room
//! has no signal, the dongle has to move somewhere that does — a Raspberry Pi
//! by a window running `rtl_tcp` — and the laptop has to read it over the
//! network instead. `fm-single` and `am-single` read raw bytes on stdin and
//! know nothing about where they came from, so the only missing piece is
//! something that speaks `rtl_tcp` and writes those same bytes. This is it.
//!
//! The audience sees no difference: the file on the slide is unchanged, and the
//! only thing that moved is which process is producing the bytes.
//!
//! The `rtl_tcp` protocol is small enough to write out in full:
//!
//!   - On connect the server sends 12 bytes: the magic "RTL0", then the tuner
//!     type and the number of gain settings, both big-endian u32.
//!   - The client sends 5-byte commands: one command byte, then a big-endian
//!     u32 argument.
//!   - Everything the server sends after the header is IQ: unsigned 8-bit
//!     samples, interleaved I, Q, I, Q — byte for byte what `rtl_sdr -` writes.
//!
//! That last line is the whole reason this is a 300-line program and not a
//! rewrite of the receiver.

use std::env;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process;
use std::time::{Duration, Instant};

// rtl_tcp command bytes. Only the four we need are named.
const CMD_SET_FREQ: u8 = 0x01;
const CMD_SET_SAMPLE_RATE: u8 = 0x02;
const CMD_SET_GAIN_MODE: u8 = 0x03; // 0 = auto, 1 = manual
const CMD_SET_GAIN: u8 = 0x04; // tenths of a dB, manual mode only

const DEFAULT_PORT: u16 = 1234;

/// The rate the single-file receivers are compiled for. `fm-single` and
/// `am-single` both hardcode `SDR_RATE = 960_000`, and nothing tells them
/// otherwise, so asking the far end for a different rate would play the audio
/// at the wrong speed with nothing in the output to say so.
const DEFAULT_SAMPLE_RATE: u32 = 960_000;

/// How long to wait for the socket before calling the link dead. At 960 kS/s
/// the far end owes us 1.92 MB every second; five seconds of total silence is
/// not jitter, it is a Pi that has gone away, and saying so beats hanging.
const READ_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Read-ahead held here before the first byte reaches the receiver.
///
/// A live pipeline has no slack: the receiver consumes samples exactly as fast
/// as they arrive, so any network stall is a hole in the audio. Buffering half
/// a second up front and then releasing it lets the receiver's own audio ring
/// run half a second ahead of the socket, which is what a WiFi hiccup eats
/// instead of the sound. The cost is half a second of extra latency at start.
const DEFAULT_PREBUFFER_MS: u64 = 500;

const CHUNK: usize = 65_536;

/// Ignored at the start of a probe. `rtl_tcp` retunes and spins up its buffers
/// when the commands arrive, so the first second or two carries less than the
/// steady-state rate through no fault of the network. Counting it made the gate
/// report a worst-second dip that streaming for a minute did not reproduce.
const PROBE_WARMUP: Duration = Duration::from_secs(3);

struct Args {
    host: String,
    port: u16,
    frequency_hz: u32,
    sample_rate: u32,
    gain: Gain,
    prebuffer_ms: u64,
    stats: bool,
    probe_secs: Option<u64>,
}

enum Gain {
    Auto,
    Manual(i32), // tenths of a dB
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("iq-tcp: {msg}");
            usage();
            process::exit(2);
        }
    };

    if let Err(e) = run(&args) {
        eprintln!("iq-tcp: {e}");
        process::exit(1);
    }
}

fn usage() {
    eprintln!(
        "usage: iq-tcp --host HOST [--port {DEFAULT_PORT}] --frequency MHZ
               [--sample-rate {DEFAULT_SAMPLE_RATE}] [--gain DB|auto]
               [--prebuffer-ms {DEFAULT_PREBUFFER_MS}] [--stats] [--probe SECONDS]

  Reads IQ from a remote rtl_tcp and writes it to stdout in rtl_sdr's format.

    iq-tcp --host radio.local --frequency 97.7 | fm-single
    iq-tcp --host radio.local --frequency 119.9 --gain 40 | am-single

  --probe SECONDS measures the link instead of streaming: it reports the
  delivered rate against the rate the receiver needs and exits non-zero if the
  link cannot keep up. Run it before the session, not during it."
    );
}

fn parse_args() -> Result<Args, String> {
    let mut host: Option<String> = None;
    let mut port = DEFAULT_PORT;
    let mut frequency: Option<f64> = None;
    let mut sample_rate = DEFAULT_SAMPLE_RATE;
    let mut gain = Gain::Auto;
    let mut prebuffer_ms = DEFAULT_PREBUFFER_MS;
    let mut stats = false;
    let mut probe_secs = None;

    let argv: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let flag = argv[i].as_str();
        // Every flag here takes a value, so fetch it once and complain once.
        let mut value = || -> Result<String, String> {
            i += 1;
            argv.get(i)
                .cloned()
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag {
            "--host" | "-H" => host = Some(value()?),
            "--port" | "-p" => {
                port = value()?
                    .parse()
                    .map_err(|_| "--port must be a port number".to_string())?
            }
            "--frequency" | "-f" => {
                let raw = value()?;
                let mhz: f64 = raw
                    .parse()
                    .map_err(|_| format!("--frequency must be a number of MHz, got {raw:?}"))?;
                if !(mhz > 0.0) {
                    return Err("--frequency must be positive".to_string());
                }
                frequency = Some(mhz);
            }
            "--sample-rate" | "-s" => {
                sample_rate = value()?
                    .parse()
                    .map_err(|_| "--sample-rate must be a number of Hz".to_string())?
            }
            "--gain" | "-g" => {
                let raw = value()?;
                gain = if raw == "auto" {
                    Gain::Auto
                } else {
                    let db: f64 = raw
                        .parse()
                        .map_err(|_| format!("--gain must be a number of dB or \"auto\", got {raw:?}"))?;
                    Gain::Manual((db * 10.0).round() as i32)
                };
            }
            "--prebuffer-ms" => {
                prebuffer_ms = value()?
                    .parse()
                    .map_err(|_| "--prebuffer-ms must be a number of milliseconds".to_string())?
            }
            "--stats" => stats = true,
            "--probe" => {
                probe_secs = Some(
                    value()?
                        .parse()
                        .map_err(|_| "--probe must be a number of seconds".to_string())?,
                )
            }
            "--help" | "-h" => {
                usage();
                process::exit(0);
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
        i += 1;
    }

    let host = host.ok_or("--host is required")?;
    let frequency = frequency.ok_or("--frequency is required")?;
    if sample_rate == 0 {
        return Err("--sample-rate must be non-zero".to_string());
    }

    Ok(Args {
        host,
        port,
        frequency_hz: (frequency * 1e6) as u32,
        sample_rate,
        gain,
        prebuffer_ms,
        stats,
        probe_secs,
    })
}

fn run(args: &Args) -> io::Result<()> {
    let mut stream = connect(&args.host, args.port)?;

    // The 12-byte greeting has to come off the socket before anything else, or
    // it would arrive at the receiver as four bytes of garbage IQ and two
    // nonsense samples. Harmless in FM, but it also tells us the far end really
    // is an rtl_tcp and not, say, an SSH banner from a mistyped port.
    let mut header = [0u8; 12];
    stream.read_exact(&mut header)?;
    let magic = &header[0..4];
    if magic != b"RTL0" {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{}:{} sent {:?}, not an rtl_tcp greeting — is that the right host and port?",
                args.host,
                args.port,
                String::from_utf8_lossy(magic)
            ),
        ));
    }
    let tuner_type = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
    let gain_count = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);

    // Order matters only in that the rate must be set before we start counting
    // bytes against it; rtl_tcp applies each command as it arrives.
    send(&mut stream, CMD_SET_SAMPLE_RATE, args.sample_rate)?;
    send(&mut stream, CMD_SET_FREQ, args.frequency_hz)?;
    match args.gain {
        Gain::Auto => send(&mut stream, CMD_SET_GAIN_MODE, 0)?,
        Gain::Manual(tenths) => {
            // Manual gain is ignored unless gain mode is switched off auto
            // first. Sending only the gain looks like it worked and does
            // nothing, which is a miserable thing to debug in a hotel room.
            send(&mut stream, CMD_SET_GAIN_MODE, 1)?;
            send(&mut stream, CMD_SET_GAIN, tenths as u32)?;
        }
    }

    let bytes_per_sec = args.sample_rate as u64 * 2;
    eprintln!(
        "iq-tcp: {}:{} tuner {} ({} gains), {:.4} MHz at {} S/s = {:.2} MB/s{}",
        args.host,
        args.port,
        tuner_name(tuner_type),
        gain_count,
        args.frequency_hz as f64 / 1e6,
        args.sample_rate,
        bytes_per_sec as f64 / 1e6,
        match args.gain {
            Gain::Auto => ", auto gain".to_string(),
            Gain::Manual(t) => format!(", gain {:.1} dB", t as f64 / 10.0),
        }
    );

    match args.probe_secs {
        Some(secs) => probe(&mut stream, secs, bytes_per_sec),
        None => stream_to_stdout(&mut stream, args, bytes_per_sec),
    }
}

fn connect(host: &str, port: u16) -> io::Result<TcpStream> {
    // Resolve first so a bad name is a clear error rather than a timeout, and
    // so we can use connect_timeout: the default connect can sit for over a
    // minute on a host that is simply not there, and on stage that is
    // indistinguishable from a hang.
    let mut addrs: Vec<SocketAddr> = (host, port)
        .to_socket_addrs()
        .map_err(|e| io::Error::new(ErrorKind::NotFound, format!("cannot resolve {host}: {e}")))?
        .collect();
    if addrs.is_empty() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("{host} resolved to no addresses"),
        ));
    }

    // A name can resolve to several addresses, and we have to try all of them.
    // `radio.local` over mDNS resolves to an IPv6 link-local address *and* an
    // IPv4 one, and hands back the IPv6 first. `rtl_tcp -a 0.0.0.0` listens on
    // IPv4 only, so trying just the first address failed with "connection
    // refused" against a server that was running perfectly.
    //
    // IPv4 goes first because that is what rtl_tcp almost always binds, which
    // also avoids waiting on a v6 address that cannot work.
    addrs.sort_by_key(|a| !a.is_ipv4());

    let mut last: Option<(SocketAddr, io::Error)> = None;
    for addr in &addrs {
        match TcpStream::connect_timeout(addr, CONNECT_TIMEOUT) {
            Ok(stream) => {
                stream.set_read_timeout(Some(READ_TIMEOUT))?;
                // The command bytes are tiny and want to leave immediately;
                // without this they can sit in the kernel waiting for company
                // that never comes.
                stream.set_nodelay(true)?;
                return Ok(stream);
            }
            Err(e) => last = Some((*addr, e)),
        }
    }

    let (addr, e) = last.expect("addrs is non-empty, so the loop ran at least once");
    let tried: Vec<String> = addrs.iter().map(|a| a.to_string()).collect();
    Err(io::Error::new(
        e.kind(),
        format!(
            "cannot reach rtl_tcp at {addr}: {e}. Tried {}: {}. Is rtl_tcp running, bound to \
             0.0.0.0, and is the network letting the two machines talk? Conference WiFi often \
             isolates clients from each other.",
            tried.len(),
            tried.join(", ")
        ),
    ))
}

fn send(stream: &mut TcpStream, cmd: u8, value: u32) -> io::Result<()> {
    let mut buf = [0u8; 5];
    buf[0] = cmd;
    buf[1..5].copy_from_slice(&value.to_be_bytes());
    stream.write_all(&buf)
}

/// The tuner types rtl_tcp reports, in its own numbering.
fn tuner_name(id: u32) -> &'static str {
    match id {
        1 => "E4000",
        2 => "FC0012",
        3 => "FC0013",
        4 => "FC2580",
        5 => "R820T",
        6 => "R828D",
        _ => "unknown",
    }
}

fn stream_to_stdout(stream: &mut TcpStream, args: &Args, bytes_per_sec: u64) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    // Fill the read-ahead before releasing anything downstream. The receiver
    // drains its own audio ring in real time, so handing it this lump at once
    // puts the whole prebuffer into that ring as slack.
    let prebuffer_bytes = (bytes_per_sec * args.prebuffer_ms / 1000) as usize;
    if prebuffer_bytes > 0 {
        let mut lead = vec![0u8; prebuffer_bytes];
        read_full(stream, &mut lead)?;
        if out.write_all(&lead).is_err() {
            return Ok(()); // receiver already gone; not our failure
        }
        if args.stats {
            eprintln!(
                "iq-tcp: {} ms of read-ahead buffered ({} bytes)",
                args.prebuffer_ms, prebuffer_bytes
            );
        }
    }

    let mut buf = vec![0u8; CHUNK];
    let start = Instant::now();
    let mut total: u64 = prebuffer_bytes as u64;
    let mut last_report = Instant::now();
    let mut last_total = total;

    loop {
        let n = match stream.read(&mut buf) {
            Ok(0) => {
                eprintln!("iq-tcp: rtl_tcp closed the connection after {total} bytes");
                return Ok(());
            }
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) if is_timeout(&e) => {
                return Err(io::Error::new(
                    ErrorKind::TimedOut,
                    format!(
                        "no data for {} s after {} bytes — the link or the Pi is gone",
                        READ_TIMEOUT.as_secs(),
                        total
                    ),
                ))
            }
            Err(e) => return Err(e),
        };

        // A write failure here is the receiver having exited — Ctrl-C on the
        // pipeline, most often. That is a normal end, not a failure.
        if out.write_all(&buf[..n]).is_err() {
            return Ok(());
        }
        total += n as u64;

        if args.stats && last_report.elapsed() >= Duration::from_secs(2) {
            let window = last_report.elapsed().as_secs_f64();
            let rate = (total - last_total) as f64 / window;
            eprintln!(
                "iq-tcp: {:.2} MB/s ({:.0}% of the {:.2} MB/s needed), {:.0} s elapsed",
                rate / 1e6,
                100.0 * rate / bytes_per_sec as f64,
                bytes_per_sec as f64 / 1e6,
                start.elapsed().as_secs_f64()
            );
            last_report = Instant::now();
            last_total = total;
        }
    }
}

/// Measure the link rather than use it. The question this answers is the only
/// one that matters before committing the demo to a network: can this path
/// carry 1.92 MB/s sustained, or will the audio break up?
fn probe(stream: &mut TcpStream, secs: u64, bytes_per_sec: u64) -> io::Result<()> {
    let window = Duration::from_secs(secs.max(1));
    let mut buf = vec![0u8; CHUNK];
    let began = Instant::now();

    // Per-second buckets, warm-up excluded. The average alone can look fine
    // while the audio stutters, so the shape of the series is the interesting
    // part and gets printed.
    let mut seconds: Vec<f64> = Vec::new();
    let mut measuring_from: Option<Instant> = None;
    let mut total: u64 = 0;
    let mut mark = Instant::now();
    let mut mark_total: u64 = 0;

    loop {
        let elapsed = began.elapsed();
        if elapsed >= PROBE_WARMUP + window {
            break;
        }
        let n = match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) if is_timeout(&e) => {
                eprintln!("iq-tcp: link went silent mid-probe — treat it as unusable");
                break;
            }
            Err(e) => return Err(e),
        };

        // Start counting only once the warm-up is behind us.
        if measuring_from.is_none() {
            if elapsed < PROBE_WARMUP {
                continue;
            }
            measuring_from = Some(Instant::now());
            mark = Instant::now();
            mark_total = 0;
            total = 0;
        }
        total += n as u64;

        if mark.elapsed() >= Duration::from_secs(1) {
            seconds.push((total - mark_total) as f64 / mark.elapsed().as_secs_f64());
            mark = Instant::now();
            mark_total = total;
        }
    }

    let measured = measuring_from
        .map(|t| t.elapsed().as_secs_f64())
        .unwrap_or(0.0);
    if measured <= 0.0 || total == 0 {
        println!("verdict     NO DATA — nothing arrived after {}s of warm-up", PROBE_WARMUP.as_secs());
        process::exit(1);
    }
    let average = total as f64 / measured;
    let needed = bytes_per_sec as f64;
    let worst = seconds.iter().cloned().fold(f64::INFINITY, f64::min);

    println!(
        "delivered   {:.2} MB/s average over {:.1} s (first {} s of warm-up excluded)",
        average / 1e6,
        measured,
        PROBE_WARMUP.as_secs()
    );
    if !seconds.is_empty() {
        println!("worst 1 s   {:.2} MB/s", worst / 1e6);
        let series: Vec<String> = seconds.iter().map(|r| format!("{:.2}", r / 1e6)).collect();
        println!("per second  {}", series.join(" "));
    }
    println!("needed      {:.2} MB/s", needed / 1e6);

    // A live source cannot deliver faster than realtime for long, so "keeping
    // up" means landing just under the required rate, not above it. Anything
    // materially below it is the link throttling the dongle.
    //
    // The dip rule, rather than a plain minimum: a single slow second at the
    // start of a stream is `rtl_tcp` still ramping, and it is inaudible — you
    // are not on air in the first second. A *sustained* shortfall is what
    // breaks audio. Judging on the plain minimum failed a link that delivered
    // 22 of 23 seconds at exactly the required rate, which is a false negative,
    // and a false negative here sends you to the recorded-file fallback for no
    // reason. The full per-second series is always printed, so the verdict
    // never hides anything it was computed from.
    let dips = seconds.iter().filter(|&&r| r < needed * 0.90).count();
    let stalled = seconds.iter().any(|&r| r < needed * 0.60);
    if !seconds.is_empty() {
        println!("dips <90%   {dips} of {} seconds", seconds.len());
    }
    let ok = average >= needed * 0.98 && dips <= 1 && !stalled;
    if ok {
        println!("verdict     OK — the link carries the stream");
        Ok(())
    } else {
        println!("verdict     TOO SLOW — expect dropouts; use a hardline or move closer");
        process::exit(1);
    }
}

/// `read_exact`, but reporting how far it got, because "the link died 300 ms
/// in" and "the link never opened" want different responses from a stage.
fn read_full(stream: &mut TcpStream, buf: &mut [u8]) -> io::Result<()> {
    let mut filled = 0;
    while filled < buf.len() {
        match stream.read(&mut buf[filled..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("rtl_tcp closed after {filled} of {} bytes", buf.len()),
                ))
            }
            Ok(n) => filled += n,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) if is_timeout(&e) => {
                return Err(io::Error::new(
                    ErrorKind::TimedOut,
                    format!(
                        "no data for {} s while buffering ({filled} of {} bytes)",
                        READ_TIMEOUT.as_secs(),
                        buf.len()
                    ),
                ))
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// A read timeout surfaces as `WouldBlock` on Unix and `TimedOut` on Windows.
fn is_timeout(e: &io::Error) -> bool {
    matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}
