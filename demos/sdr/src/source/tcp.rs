use crate::source::{IqSample, SdrSource};
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::net::TcpStream;

// rtl_tcp command bytes
const CMD_SET_FREQ: u8 = 0x01;
const CMD_SET_SAMPLE_RATE: u8 = 0x02;
const CMD_SET_GAIN_MODE: u8 = 0x03; // 0 = auto, 1 = manual
const CMD_SET_GAIN: u8 = 0x04; // tenths of a dB, manual mode only
const CMD_SET_DIRECT_SAMPLING: u8 = 0x09;

pub struct TcpSource {
    stream: TcpStream,
    sample_rate: u32,
}

impl TcpSource {
    pub fn connect(host: &str, port: u16) -> Result<Self> {
        let stream = TcpStream::connect((host, port))
            .with_context(|| format!("connecting to rtl_tcp at {host}:{port}"))?;

        // rtl_tcp sends a 12-byte dongle info header on connect
        let mut header = [0u8; 12];
        (&stream).read_exact(&mut header)?;

        Ok(Self {
            stream,
            sample_rate: 2_048_000, // rtl_tcp default
        })
    }

    fn send_command(&mut self, cmd: u8, value: u32) -> Result<()> {
        let mut buf = [0u8; 5];
        buf[0] = cmd;
        buf[1..5].copy_from_slice(&value.to_be_bytes());
        self.stream
            .write_all(&buf)
            .context("sending rtl_tcp command")?;
        Ok(())
    }
}

impl SdrSource for TcpSource {
    fn read(&mut self, buf: &mut [IqSample]) -> Result<usize> {
        let mut raw = vec![0u8; buf.len() * 2];
        self.stream.read_exact(&mut raw)?;

        for i in 0..buf.len() {
            buf[i] = IqSample::new(
                (raw[i * 2] as f32 - 127.5) / 127.5,
                (raw[i * 2 + 1] as f32 - 127.5) / 127.5,
            );
        }

        Ok(buf.len())
    }

    fn set_frequency(&mut self, freq_hz: u32) -> Result<()> {
        self.send_command(CMD_SET_FREQ, freq_hz)
    }

    fn set_sample_rate(&mut self, rate: u32) -> Result<()> {
        self.send_command(CMD_SET_SAMPLE_RATE, rate)?;
        self.sample_rate = rate;
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Without this the trait's default no-op applied, so `--gain` on a TCP
    /// source did nothing and said nothing — the receiver ran at whatever gain
    /// `rtl_tcp` was started with while reporting the gain you asked for.
    ///
    /// Manual gain also needs the mode command first. Sending only the gain
    /// looks like it worked and has no effect.
    fn set_gain(&mut self, gain: Option<i32>) -> Result<()> {
        match gain {
            Some(tenths) => {
                self.send_command(CMD_SET_GAIN_MODE, 1)?;
                self.send_command(CMD_SET_GAIN, tenths as u32)
            }
            None => self.send_command(CMD_SET_GAIN_MODE, 0),
        }
    }

    fn set_direct_sampling(&mut self, mode: u32) -> Result<()> {
        self.send_command(CMD_SET_DIRECT_SAMPLING, mode)
    }
}
