use num_complex::Complex;

pub type IqSample = Complex<f32>;

/// Convert raw RTL-SDR u8 pairs to IQ samples.
/// RTL-SDR outputs unsigned 8-bit I/Q interleaved: [I0, Q0, I1, Q1, ...]
/// We center around zero and normalize to [-1.0, 1.0].
pub fn bytes_to_iq(raw: &[u8]) -> Vec<IqSample> {
    raw.chunks_exact(2)
        .map(|pair| {
            IqSample::new(
                (pair[0] as f32 - 127.5) / 127.5,
                (pair[1] as f32 - 127.5) / 127.5,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_iq_center() {
        let raw = [128, 128];
        let samples = bytes_to_iq(&raw);
        assert_eq!(samples.len(), 1);
        assert!((samples[0].re - 0.00392).abs() < 0.01);
        assert!((samples[0].im - 0.00392).abs() < 0.01);
    }

    #[test]
    fn test_bytes_to_iq_extremes() {
        let raw = [0, 255];
        let samples = bytes_to_iq(&raw);
        assert!(samples[0].re < -0.99);
        assert!(samples[0].im > 0.99);
    }
}
