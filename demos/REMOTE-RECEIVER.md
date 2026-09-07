# The remote receiver: running the AM/FM demos off a Pi

**What this is for.** The AM/FM demos assume a dongle plugged into the presentation
laptop and a signal reaching it. A main stage is usually the worst room in the
building for that: no window, a lot of metal, and a lighting rig throwing hash across
the whole VHF band. This path moves the dongle somewhere the signal actually arrives —
a Raspberry Pi by a window — and brings the samples back over the network.

It sits between the live-on-stage demo and the recorded-file fallback:

| | what runs | needs | falls back to |
|---|---|---|---|
| 1 | `task fm-single` / `task am-single` | dongle on the laptop, signal in the room | 2 |
| 2 | `task fm-remote` / `task am-remote` | Pi with signal, network to it | 3 |
| 3 | `task fm-file` / `task am-file` | a recorded `.iq` on disk | nothing; it always works |

Tier 3 is still the one that cannot fail, and it is still what you reach for if
anything is uncertain at the podium. Tier 2 exists so that "no signal on stage" does
not immediately cost you a *live* demo.

## What actually changes

Nothing in the receiver. `fm-single` and `am-single` read raw 8-bit IQ on stdin and
know nothing about where it came from — that is the whole reason this works. The only
substitution is which process produces those bytes:

```
local:   rtl_sdr -f 97700000 -s 960000 -g 30 -  | fm-single
remote:  iq-tcp --host radio.local -f 97.7 -g 30 | fm-single
file:    iq-play fm.iq                           | fm-single
```

The file on the "The Whole Radio" slide is byte-for-byte the same file in all three
cases. If you end up on stage explaining why the receiver is remote, that is the point
worth making, not an apology.

`iq-tcp` speaks the `rtl_tcp` protocol: read the 12-byte greeting, send 5-byte
commands for sample rate, frequency and gain, then relay everything after that
straight to stdout. It is about 300 lines with no dependencies, for the same reason
`iq-play` has none — stage tooling must never be the thing that fails.

## The bandwidth question, answered up front

960 kS/s of 8-bit I and Q is **1.92 MB/s, about 15.4 Mbit/s, sustained, in one
direction, for as long as the demo runs.** There is no compression anywhere in
`rtl_tcp`; it is raw ADC output.

That is comfortable on a hardline or clean 5 GHz WiFi, and marginal on a congested
2.4 GHz conference network. Measure it, don't assume it:

```bash
task link-check HOST=radio.local SECS=10
```

It reports the delivered rate, the worst one-second window (which is what the ear
hears — an average can look fine while the audio stutters), and exits non-zero if the
link cannot keep up. Run it at the venue, on venue infrastructure, during the break.

**Note the tension with the ADS-B story.** The deck says out loud that ADS-B at
2.4 MS/s is ~4.8 MB/s and is *not* going over conference WiFi, which is why the Pi
runs the whole of `skyward` and serves HTTP. That is still true. This path is half the
sample rate and a quarter the ambition — one audio demo, briefly — but it is the same
kind of link, and if `link-check` says no, believe it. Do not use this path as a
counter-example on stage to a slide that says raw IQ does not cross a network.

## Setting up the Pi

Once, at home:

```bash
sudo apt install rtl-sdr
# The DVB-T driver grabs the dongle at plug-in and never lets go.
echo 'blacklist dvb_usb_rtl28xxu' | sudo tee /etc/modprobe.d/blacklist-rtl.conf
sudo reboot
rtl_test             # should name the tuner, not "No supported devices found"
```

**Not `rtl_test -t`.** The tuner-range test enables direct sampling and then
aborts with "No E4000 tuner found", leaving it enabled in the dongle's
registers — where it survives across program runs. Direct sampling bypasses the
tuner completely, so every frequency you tune afterwards reads as noise, with
nothing to say why. `sdr/src/source/usb.rs` clears it defensively on open for
exactly this reason; `rtl_tcp` does not. Plain `rtl_test` names the tuner
without touching it.

Then serve it. `-a 0.0.0.0` is not optional: `rtl_tcp` binds loopback by default and is
then reachable only from the Pi itself.

```bash
rtl_tcp -a 0.0.0.0 -s 960000 -f 97700000 -g 30
```

From the laptop, `task pi-serve HOST=radio.local` runs exactly that line over ssh and
leaves it in the foreground of its own terminal, which is the easiest thing to watch
and the easiest thing to restart.

### If you are preparing the card from scratch

Raspberry Pi OS ships `ssh.service` **disabled**, and current images configure
first boot through cloud-init (`user-data` / `network-config` on the boot
partition) rather than the older `custom.toml`. cloud-init will install your
authorised key and never turn the service on, so the Pi comes up pingable and
impossible to log into — and if the account has no password, a keyboard and
monitor will not save you either. Two ways out, both cheap:

- put an empty file named `ssh` on the boot partition (`sshswitch.service` ships
  enabled and looks for `/boot/firmware/ssh`), or
- put `- [ systemctl, enable, --now, ssh ]` in the cloud-init `runcmd`.

Do one of them *before* first boot. Note that cloud-init caches `instance_id`
and will not re-run `runcmd` on an already-initialised system, so fixing
`user-data` after the fact does nothing; the `ssh` file is the recovery path.

For something that survives a reboot and a power cycle unattended, `/etc/systemd/system/rtl-tcp.service`:

```ini
[Unit]
Description=rtl_tcp IQ server
After=network-online.target
Wants=network-online.target

[Service]
ExecStart=/usr/bin/rtl_tcp -a 0.0.0.0 -s 960000 -f 97700000 -g 30
Restart=always
RestartSec=2
User=pi

[Install]
WantedBy=multi-user.target
```

`sudo systemctl enable --now rtl-tcp`.

Three things about `rtl_tcp` worth knowing before they surprise you:

- **`[R82XX] PLL not locked!` at startup is normal.** It appears while the tuner
  settles during the initial tune and is followed by `Tuned to … Hz`. Verified
  irrelevant: every clean measurement in this document was taken from a server
  that logged it. Do not go debugging it at a venue.

- **It serves one client at a time.** A second connection does not get a second
  stream. If `iq-tcp` says the link is refused and the Pi looks healthy, something
  else is already connected — an old `iq-tcp` you forgot to kill, most likely.
- **Frequency, rate and gain set on the command line are only defaults.** `iq-tcp`
  sends its own on connect, so `task fm-remote FREQ=…` wins over whatever the service
  was started with. You do not need to restart the Pi to change station.

## Tailscale, and why it is not just convenience

Conference WiFi commonly isolates clients from each other, so two machines can
both be online and still be unable to exchange a packet. That is the failure
this path is most exposed to, and it is not fixable from a stage. Tailscale
routes around it: the Pi and the laptop meet on the tailnet regardless of what
the local network permits between them.

It is installed by the cloud-init `runcmd` on the card. Authorising it needs one
interactive step, deliberately — the alternative is an auth key, and an auth key
is a tailnet-joining secret that would have to sit in plaintext on a FAT
partition:

```bash
ssh pi@radio.local
sudo tailscale up --hostname radio    # prints a URL; open it once
```

After that, `task fm-remote HOST=radio` works by MagicDNS from anywhere the
laptop can reach the tailnet.

**Check that it is direct, not relayed, before trusting it with the stream:**

```bash
tailscale ping radio
```

`pong … via 192.168.x.x:41641` or any real address means a direct peer-to-peer
path, and throughput is essentially native. `pong … via DERP` means Tailscale
could not punch through and is relaying through Tailscale's infrastructure.
DERP is built for keeping connections alive, not for carrying 15 Mbit/s of raw
IQ; if you see it, `link-check` will tell you the rest, and the answer will be
tier 3.

Measured at home on 2026-09-06: direct, and the tailnet was marginally *steadier*
than the raw LAN — worst second 1.91 MB/s versus 1.89 — presumably WireGuard
pacing smoothing the bursts. Encryption overhead is irrelevant here; a Pi 4 does
WireGuard an order of magnitude faster than 15 Mbit/s.

## Prefer Ethernet, and what it actually buys

A hardline removes the three things WiFi exposes this path to at once: the
throughput margin, the captive portal, and client isolation. Measured at home on
2026-09-06, 25-second samples with warm-up excluded:

| path | average | worst second | spread | RTT |
|---|---|---|---|---|
| Ethernet (gigabit) | 1.92 MB/s | **1.91** | 1.91–1.93 | 1.0 ms |
| 5 GHz WiFi | 1.92 MB/s | 1.89 | 1.89–1.95 | 100 ms avg, **14–186** |

Both pass. The difference is not the average — a live source cannot exceed
realtime, so the average is pinned either way — it is the spread and the latency
jitter. WiFi's 14–186 ms RTT is what produced its wider per-second range, and it
is the thing a busier venue network will make worse.

**No configuration is needed to prefer it.** NetworkManager's default route
metrics are 100 for Ethernet and 600 for WiFi, so plugging a cable in takes over
automatically while WiFi stays up as a standby, and `radio.local` follows to the
wired address on its own. Two properties are worth setting on the wired profile
anyway, for the case where the venue drop is not what you were promised:

```bash
sudo nmcli connection modify netplan-eth0 ipv4.link-local fallback
sudo nmcli connection modify netplan-eth0 ipv4.dhcp-timeout 15
```

`link-local fallback` covers a dead port or an unmanaged switch with no DHCP: the
Pi self-assigns a `169.254.x.x` address, macOS does the same, and mDNS works over
link-local — so **a cable straight from laptop to Pi works with no network
involved at all.** That is the one topology that needs no venue cooperation
whatsoever, and it is worth knowing you have it.

### What happens when the cable is kicked

Tested by disconnecting `eth0` mid-stream:

- The Pi recovers on its own. The default route moves to WiFi within seconds,
  `rtl-tcp.service` stays active because it binds `0.0.0.0` rather than an
  address, and the tailnet address does not change. On reconnect, Ethernet
  resumes as the preferred route.
- **The stream in flight does not survive.** It delivered 9.7 s of audio and then
  failed with `no data for 5 s — the link or the Pi is gone`. Tailscale reroutes
  *new* connections, not this one, and waiting longer would not have rescued it:
  `rtl_tcp` goes on producing 1.92 MB/s into a buffer it eventually drops, so
  there is no backlog worth resuming after a multi-second gap.

That is the right behaviour for a stage — you learn in five seconds rather than
staring at silence — but it means a kicked cable ends the demo. Restarting the
task reconnects fine. If it happens live, go to tier 3 and do not debug.

### Which HOST to use at the venue

`HOST=radio` over the tailnet works in every topology, including the likely one
where the Pi is on the venue's wired network and the laptop is on venue WiFi —
different subnets, across which `radio.local` mDNS often does not resolve. Use it
as the default. `HOST=radio.local` is the same-network shortcut and avoids
depending on the tailnet at all; at home both measured identically. Whichever you
use, `link-check` it first.

## Before you leave home: give the Pi a second network

The card was configured with one SSID. At a venue that SSID does not exist, so
the Pi joins nothing, and because `wlan0` is marked `optional: false` it waits at
boot rather than coming up without a network. There is then no way in — no SSH,
and no console either if the account has no password.

cloud-init will not help: it caches `instance_id` and will not re-run
`network-config` on an already-initialised system. Add networks through
NetworkManager on the Pi instead, over SSH, while you are still on a network that
works:

```bash
ssh pi@radio.local
sudo nmcli connection add type wifi con-name venue ssid "VENUE_SSID" \
  wifi-sec.key-mgmt wpa-psk wifi-sec.psk "VENUE_PASSWORD"
sudo nmcli connection modify venue connection.autoconnect yes
nmcli connection show          # confirm both networks are listed
```

Add a **phone hotspot** the same way, as the one network you control completely.
It is the answer to the three venue conditions that no amount of preparation
fixes from the floor:

- a **captive portal** — the Pi cannot click a web form,
- **WPA2-Enterprise** with a username, which the PSK setup above does not cover,
- **client isolation**, which Tailscale already handles.

With Tailscale, the Pi does not need to be on the same network as the laptop at
all: Pi on your hotspot and laptop on venue WiFi still meet on the tailnet. Be
aware what that costs, though — 1.92 MB/s is **115 MB per minute** of cellular
upload, so a two-minute demo is about 230 MB, and the connection will likely be
relayed rather than direct. Treat it as the last rung before tier 3, not a plan.

## On the day

```bash
task alias                                  # before the session, never on stage
task link-check HOST=radio.local SECS=10    # gate: non-zero exit means don't
task fm-remote HOST=radio.local FREQ=97.7 GAIN=30
task am-remote HOST=radio.local FREQ=119.9 GAIN=40
```

Gains carry over from the local tasks and for the same reasons: broadcast FM is strong
enough to rail the ADC, aviation AM is weak and intermittent. They are the Pi's tuner
gain, set over the socket, so they can be changed by restarting the pipeline — no trip
upstairs.

`iq-tcp` buffers 500 ms of read-ahead before the first byte reaches the receiver. That
half-second of slack is what a WiFi hiccup eats instead of the audio, and it costs half
a second of extra latency at start. `--prebuffer-ms` moves it; more is more robust and
later.

## Rehearsing it without a Pi

`rtl-tcp-fake` serves a recorded `.iq` file with the same greeting, the same commands
and the same pacing as the real thing, so the entire laptop half can be exercised on a
plane:

```bash
task fake-pi FILE=fm.iq                 # terminal 1
task fm-remote HOST=127.0.0.1 FREQ=101.7 GAIN=20   # terminal 2
```

It is a test fixture, not a radio. Nothing on stage should ever point at it — if you
can run `rtl-tcp-fake`, you can run `task fm-file`, which is one process shorter and is
tier 3.

`--rate` throttles it, which is how the "TOO SLOW" verdict from `link-check` was
tested: `task fake-pi FILE=fm.iq` with `bin/rtl-tcp-fake fm.iq --rate 1000000`.

## What the errors mean

| message | what to do |
|---|---|
| `cannot reach rtl_tcp at …: Connection refused` | `rtl_tcp` is not running, or is bound to 127.0.0.1 |
| `cannot reach rtl_tcp at …: Operation timed out` | client isolation on the WiFi, or a firewall. Not fixable from a stage — go to tier 3 |
| `sent "SSH-", not an rtl_tcp greeting` | right host, wrong port |
| `no data for 5 s … the link or the Pi is gone` | the stream died mid-demo. Go to tier 3; do not debug |
| `verdict TOO SLOW` | the network will not carry it. Hardline, or tier 3 |

The trigger criterion, decided now rather than at the podium: **if `link-check` fails
during the break, the AM/FM demos run from file and the Pi is not mentioned.** Do not
try tier 2 live on a link that has not passed.

## Verification status

Verified on 2026-09-06 against `rtl-tcp-fake`, with no hardware:

- `iq-tcp` strips the greeting and reproduces the served file byte-for-byte,
  and sends the tuning commands in the right order (gain mode before gain, or
  manual gain is silently ignored).
- `task fm-remote` and `task fm-file` fed from the same `fm.iq` produce
  **bit-identical audio** — 622,575 of 622,575 samples equal, max difference
  0.000e+00. The remote path is not "close enough"; it is the same numbers.
- Connect failure, wrong protocol, missing arguments, and both `link-check`
  verdicts including a throttled link.

Verified the same day **on the real Pi, over WiFi, with the dongle attached**:

- `rtl_test` names a Rafael Micro R820T with 29 gain values, and
  `dvb_usb_rtl28xxu` is not loaded, so the blacklist works. The fake had
  advertised tuner type 5 and 29 gains, so it was faithful.
- `rtl_tcp` serves on `0.0.0.0:1234` from a systemd unit across a reboot.
- **Sustained throughput, 25 s samples, warm-up excluded: 1.92 MB/s average on
  both paths** against the 1.92 MB/s requirement — LAN over 5 GHz WiFi worst
  second 1.89, tailnet worst second 1.91, zero sub-90% seconds out of 22 on
  either. The Pi's link negotiates 270 Mbit/s at signal 74/100, so 15.4 Mbit/s
  is not straining it; the rate is set by the dongle, which is what a live
  source should look like.
- Live FM from the Pi at 101.7 is statistically indistinguishable from the
  known-good `fm.iq` capture: rms 0.1107 vs 0.1132, crest 2.52 vs 2.47, 81.7%
  vs 82.4% of energy below 1 kHz, and zero clipped samples at gain 20.

Two bugs that only real hardware could find, both fixed:

- **`iq-tcp` tried only the first resolved address.** `radio.local` resolves
  over mDNS to an IPv6 link-local address *and* an IPv4 one, IPv6 first, and
  `rtl_tcp -a 0.0.0.0` is IPv4-only — so a perfectly healthy server refused the
  connection. It now tries every address, IPv4 first.
- **`--probe` counted `rtl_tcp`'s ramp-up, and then judged too harshly.** The
  first seconds after the tuning commands carry less than steady state through
  no fault of the network. Worse, judging on the plain worst second failed a
  tailnet that delivered 22 of 23 seconds at exactly the required rate — a false
  negative, which is the expensive direction: it sends you to tier 3 for
  nothing. The probe now excludes a 3-second warm-up, prints the per-second
  series, and judges on the average plus a tolerance of one slow second rather
  than on the minimum.

  Both halves of that rule are load-bearing, and were checked against the fake:
  a link throttled to 1.0 MB/s fails on dips (11 of 11), while one throttled to
  1.80 MB/s — a sustained 6% shortfall that never dips below 90% — fails on the
  average instead. Neither rule alone catches both shapes.

- Live FM over the **tailnet**, addressed as `HOST=radio` by MagicDNS: locked
  station, crest 2.33, 1.2% of energy above 5 kHz, zero clipping.
- **Survives a reboot with nobody logged in.** Back on SSH in ~30 s, `rtl_tcp`
  claims the dongle by itself, `tailscaled` reconnects with the same tailnet
  address, `eth0` returns as the preferred route, and the `nmcli` properties above
  persist rather than being regenerated by netplan. Post-reboot Ethernet
  `link-check` was the steadiest run measured: 13 of 13 seconds at exactly
  1.92 MB/s.

### The honest read on the headroom

A live source cannot run *faster* than realtime, so a healthy link looks exactly
like this: pinned at the requirement, with no visible reserve. That is not the
same as having headroom. Nothing here has been measured against a venue network,
which will have far more contention than a home mesh with two machines on it.

So at the venue: run `link-check` and **read the per-second series, not the
verdict**. The verdict compresses the evidence into one word; the series is the
evidence. Repeated seconds below about 1.85 mean tier 3, whatever the verdict
says.
