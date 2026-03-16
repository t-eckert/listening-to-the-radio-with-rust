use crate::source::{IqSample, SdrSource};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::{Duration, Instant};

pub struct FileSource {
    file: File,
    sample_rate: u32,
    last_read: Instant,
}

impl FileSource {
    pub fn open(path: &str, sample_rate: u32) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("opening IQ file: {path}"))?;
        Ok(Self {
            file,
            sample_rate,
            last_read: Instant::now(),
        })
    }
}

impl SdrSource for FileSource {
    fn read(&mut self, buf: &mut [IqSample]) -> Result<usize> {
        // Pace output to simulate real-time streaming
        let expected_duration =
            Duration::from_secs_f64(buf.len() as f64 / self.sample_rate as f64);
        let elapsed = self.last_read.elapsed();
        if elapsed < expected_duration {
            thread::sleep(expected_duration - elapsed);
        }
        self.last_read = Instant::now();

        // Read raw u8 pairs (I, Q) and convert to f32
        let mut raw = vec![0u8; buf.len() * 2];
        let bytes_read = self.file.read(&mut raw)?;
        let samples = bytes_read / 2;

        for i in 0..samples {
            buf[i] = IqSample::new(
                (raw[i * 2] as f32 - 127.5) / 127.5,
                (raw[i * 2 + 1] as f32 - 127.5) / 127.5,
            );
        }

        Ok(samples)
    }

    fn set_frequency(&mut self, _freq_hz: u32) -> Result<()> {
        // No-op for file source
        Ok(())
    }

    fn set_sample_rate(&mut self, rate: u32) -> Result<()> {
        self.sample_rate = rate;
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
