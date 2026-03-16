use num_complex::Complex;
use std::f32::consts::PI;

/// Goertzel algorithm for detecting a single frequency in a signal.
///
/// More efficient than FFT when you only care about one or two frequencies.
pub struct GoertzelDetector {
    coeff: f32,
    target_freq: f32,
    #[allow(dead_code)]
    sample_rate: f32,
}

impl GoertzelDetector {
    pub fn new(target_freq: f32, sample_rate: f32) -> Self {
        let normalized = 2.0 * PI * target_freq / sample_rate;
        let coeff = 2.0 * normalized.cos();
        Self {
            coeff,
            target_freq,
            sample_rate,
        }
    }

    /// Compute the magnitude (power) of the target frequency in the buffer.
    pub fn power(&self, samples: &[f32]) -> f32 {
        let mut s0: f32;
        let mut s1 = 0.0f32;
        let mut s2 = 0.0f32;

        for &sample in samples {
            s0 = sample + self.coeff * s1 - s2;
            s2 = s1;
            s1 = s0;
        }

        // Magnitude squared
        s1 * s1 + s2 * s2 - self.coeff * s1 * s2
    }

    /// Detect whether the target frequency is present above a threshold.
    pub fn detect(&self, samples: &[f32], threshold: f32) -> bool {
        self.power(samples) > threshold
    }

    pub fn target_freq(&self) -> f32 {
        self.target_freq
    }
}

/// Frequency-shift a complex signal by `shift_hz`.
///
/// Multiplies the signal by e^(j·2π·f_shift·t), effectively moving it
/// in frequency.
pub fn frequency_shift(
    samples: &mut [Complex<f32>],
    shift_hz: f32,
    sample_rate: f32,
    phase: &mut f32,
) {
    let phase_step = 2.0 * PI * shift_hz / sample_rate;
    for sample in samples.iter_mut() {
        let rotator = Complex::new(phase.cos(), phase.sin());
        *sample *= rotator;
        *phase += phase_step;
        // Keep phase in [0, 2π) to avoid precision loss
        if *phase > 2.0 * PI {
            *phase -= 2.0 * PI;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goertzel_detects_tone() {
        let sample_rate = 8000.0;
        let target = 1000.0;
        let detector = GoertzelDetector::new(target, sample_rate);

        // Generate a pure 1000 Hz tone
        let n = 256;
        let samples: Vec<f32> = (0..n)
            .map(|i| (2.0 * PI * target * i as f32 / sample_rate).sin())
            .collect();

        let power = detector.power(&samples);
        assert!(power > 100.0, "Should detect 1000 Hz tone, got power {power}");

        // Generate a 2000 Hz tone — should NOT trigger
        let other: Vec<f32> = (0..n)
            .map(|i| (2.0 * PI * 2000.0 * i as f32 / sample_rate).sin())
            .collect();
        let power_other = detector.power(&other);
        assert!(
            power > power_other * 10.0,
            "Target tone should be much stronger than off-frequency"
        );
    }
}
