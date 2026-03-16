/// Single-pole IIR de-emphasis filter.
///
/// FM broadcasting uses a 75 μs pre-emphasis in North America (50 μs in Europe).
/// This filter compensates by rolling off high frequencies.
pub struct DeEmphasisFilter {
    alpha: f32,
    prev: f32,
}

impl DeEmphasisFilter {
    /// Create a de-emphasis filter.
    /// - `tau`: time constant in seconds (75e-6 for North America, 50e-6 for Europe)
    /// - `sample_rate`: audio sample rate in Hz
    pub fn new(tau: f32, sample_rate: f32) -> Self {
        // α = 1 - e^(-1/(τ·fs))
        let alpha = 1.0 - (-1.0 / (tau * sample_rate)).exp();
        Self { alpha, prev: 0.0 }
    }

    /// Process a buffer of audio samples in-place.
    pub fn process(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            self.prev += self.alpha * (*sample - self.prev);
            *sample = self.prev;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dc_passes_through() {
        let mut filter = DeEmphasisFilter::new(75e-6, 48000.0);
        let mut buf = vec![1.0f32; 1000];
        filter.process(&mut buf);
        // After settling, output should approach 1.0
        assert!((buf[999] - 1.0).abs() < 0.01);
    }
}
