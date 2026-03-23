use crate::source::{IqSample, SdrSource};
use anyhow::{Context, Result};
use rtl_sdr_rs::{DirectSampleMode, RtlSdr, TunerGain};

pub struct UsbSource {
    dev: RtlSdr,
    sample_rate: u32,
}

// SAFETY: UsbSource is only used from a single thread (the main processing loop).
// RtlSdr is !Send because it holds Box<dyn Tuner> without a Send bound, but the
// underlying USB handle is not shared across threads.
unsafe impl Send for UsbSource {}

impl UsbSource {
    pub fn open(index: u32) -> Result<Self> {
        let mut dev =
            RtlSdr::open_with_index(index as usize).context("opening RTL-SDR device")?;

        let sample_rate = 2_048_000;
        dev.set_sample_rate(sample_rate)
            .context("setting sample rate")?;

        dev.set_tuner_gain(TunerGain::Auto)
            .context("setting auto gain")?;

        dev.reset_buffer().context("resetting buffer")?;

        Ok(Self { dev, sample_rate })
    }
}

impl Drop for UsbSource {
    fn drop(&mut self) {
        let _ = self.dev.close();
    }
}

impl SdrSource for UsbSource {
    fn read(&mut self, buf: &mut [IqSample]) -> Result<usize> {
        let mut raw = vec![0u8; buf.len() * 2];
        let bytes_read = self.dev.read_sync(&mut raw).context("USB read")?;
        let samples = bytes_read / 2;

        for i in 0..samples {
            buf[i] = IqSample::new(
                (raw[i * 2] as f32 - 127.5) / 127.5,
                (raw[i * 2 + 1] as f32 - 127.5) / 127.5,
            );
        }

        Ok(samples)
    }

    fn set_frequency(&mut self, freq_hz: u32) -> Result<()> {
        self.dev
            .set_center_freq(freq_hz)
            .context("setting frequency")?;
        Ok(())
    }

    fn set_sample_rate(&mut self, rate: u32) -> Result<()> {
        self.dev
            .set_sample_rate(rate)
            .context("setting sample rate")?;
        self.sample_rate = rate;
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn set_gain(&mut self, gain: Option<i32>) -> Result<()> {
        match gain {
            Some(g) => self.dev.set_tuner_gain(TunerGain::Manual(g)),
            None => self.dev.set_tuner_gain(TunerGain::Auto),
        }
        .context("setting gain")?;
        Ok(())
    }

    fn set_direct_sampling(&mut self, mode: u32) -> Result<()> {
        let ds_mode = match mode {
            0 => DirectSampleMode::Off,
            1 => DirectSampleMode::On,
            2 => DirectSampleMode::OnSwap,
            _ => DirectSampleMode::Off,
        };
        self.dev
            .set_direct_sampling(ds_mode)
            .context("setting direct sampling mode")?;
        // Reset the buffer after changing sampling mode to flush stale data
        self.dev.reset_buffer().context("resetting buffer after direct sampling")?;
        Ok(())
    }
}
