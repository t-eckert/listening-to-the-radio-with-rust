mod file;
mod tcp;
#[cfg(feature = "usb")]
mod usb;

pub use file::FileSource;
pub use tcp::TcpSource;
#[cfg(feature = "usb")]
pub use usb::UsbSource;

use anyhow::{bail, Result};
use num_complex::Complex;

pub type IqSample = Complex<f32>;

pub trait SdrSource: Send {
    /// Read a buffer of IQ samples. Returns the number of samples written.
    fn read(&mut self, buf: &mut [IqSample]) -> Result<usize>;

    /// Set the center frequency in Hz.
    fn set_frequency(&mut self, freq_hz: u32) -> Result<()>;

    /// Set the sample rate in Hz.
    fn set_sample_rate(&mut self, rate: u32) -> Result<()>;

    /// Get the current sample rate.
    fn sample_rate(&self) -> u32;

    /// Set manual gain in tenths of dB (e.g., 400 = 40.0 dB). Pass None for auto gain.
    fn set_gain(&mut self, _gain: Option<i32>) -> Result<()> {
        Ok(()) // No-op for sources that don't support gain control
    }

    /// Highest manual gain the tuner supports, in tenths of dB. None if this
    /// source has no gain control or cannot report its range.
    fn max_gain(&self) -> Result<Option<i32>> {
        Ok(None)
    }

    /// Enable direct sampling mode for receiving below ~24 MHz.
    /// Mode: 0 = off, 1 = I-branch, 2 = Q-branch.
    fn set_direct_sampling(&mut self, _mode: u32) -> Result<()> {
        Ok(()) // No-op for sources that don't support direct sampling
    }
}

pub enum SourceConfig {
    File { path: String, sample_rate: u32 },
    Tcp { host: String, port: u16 },
    #[cfg(feature = "usb")]
    Usb { index: u32 },
}

pub fn open_source(config: SourceConfig) -> Result<Box<dyn SdrSource>> {
    match config {
        SourceConfig::File { path, sample_rate } => {
            Ok(Box::new(FileSource::open(&path, sample_rate)?))
        }
        SourceConfig::Tcp { host, port } => Ok(Box::new(TcpSource::connect(&host, port)?)),
        #[cfg(feature = "usb")]
        SourceConfig::Usb { index } => Ok(Box::new(UsbSource::open(index)?)),
    }
}

/// Parse a source string like "file:path.bin", "tcp:host:port", or "usb" / "usb:0".
pub fn parse_source_arg(s: &str, sample_rate: u32) -> Result<SourceConfig> {
    if let Some(path) = s.strip_prefix("file:") {
        Ok(SourceConfig::File {
            path: path.to_string(),
            sample_rate,
        })
    } else if let Some(rest) = s.strip_prefix("tcp:") {
        let parts: Vec<&str> = rest.rsplitn(2, ':').collect();
        if parts.len() != 2 {
            bail!("TCP source format: tcp:host:port");
        }
        let port: u16 = parts[0].parse()?;
        let host = parts[1].to_string();
        Ok(SourceConfig::Tcp { host, port })
    } else if s == "usb" || s.starts_with("usb:") {
        #[cfg(feature = "usb")]
        {
            let index = if let Some(idx) = s.strip_prefix("usb:") {
                idx.parse()?
            } else {
                0
            };
            Ok(SourceConfig::Usb { index })
        }
        #[cfg(not(feature = "usb"))]
        bail!("USB support not compiled in. Rebuild with --features usb")
    } else {
        bail!("Unknown source: {s}. Use file:PATH, tcp:HOST:PORT, or usb[:INDEX]")
    }
}
