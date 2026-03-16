use num_complex::Complex;

/// AM envelope detector.
///
/// Simply computes the magnitude of each IQ sample: |s| = sqrt(I² + Q²).
/// For AM signals, this extracts the modulating audio.
pub struct AmDemodulator;

impl AmDemodulator {
    pub fn new() -> Self {
        Self
    }

    /// Demodulate AM by computing the envelope (magnitude) of IQ samples.
    pub fn process(&self, input: &[Complex<f32>]) -> Vec<f32> {
        input.iter().map(|s| s.norm()).collect()
    }

    /// Demodulate AM and remove DC offset by subtracting the running mean.
    pub fn process_ac_coupled(&self, input: &[Complex<f32>]) -> Vec<f32> {
        let envelope: Vec<f32> = input.iter().map(|s| s.norm()).collect();
        let mean: f32 = envelope.iter().sum::<f32>() / envelope.len().max(1) as f32;
        envelope.iter().map(|&x| x - mean).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn envelope_of_carrier() {
        let demod = AmDemodulator::new();
        // Pure carrier at unit amplitude
        let input: Vec<Complex<f32>> = (0..100)
            .map(|i| {
                let phase = 2.0 * PI * i as f32 / 20.0;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect();

        let output = demod.process(&input);
        // Magnitude should be ~1.0 for all samples
        for &val in &output {
            assert!((val - 1.0).abs() < 0.01);
        }
    }
}
