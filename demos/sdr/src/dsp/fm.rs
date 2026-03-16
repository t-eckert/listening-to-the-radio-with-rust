use num_complex::Complex;

/// FM demodulator using the conjugate-multiply method.
///
/// For each pair of consecutive IQ samples, the instantaneous frequency
/// is proportional to the phase difference:
///   phase_diff = arg(sample[n] * conj(sample[n-1]))
pub struct FmDemodulator {
    prev: Complex<f32>,
    gain: f32,
}

impl FmDemodulator {
    /// Create a new FM demodulator.
    /// - `max_deviation`: max frequency deviation in Hz (e.g., 75_000 for WBFM)
    /// - `sample_rate`: sample rate of the input signal
    pub fn new(max_deviation: f32, sample_rate: f32) -> Self {
        // Normalize so that max deviation maps to [-1.0, 1.0]
        let gain = sample_rate / (2.0 * std::f32::consts::PI * max_deviation);
        Self {
            prev: Complex::new(0.0, 0.0),
            gain,
        }
    }

    /// Demodulate a buffer of IQ samples, returning audio samples.
    pub fn process(&mut self, input: &[Complex<f32>]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            let product = sample * self.prev.conj();
            let phase = product.im.atan2(product.re);
            output.push(phase * self.gain);
            self.prev = sample;
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn constant_frequency_offset() {
        // A constant frequency offset should produce a constant output
        let sample_rate = 1000.0;
        let freq_offset = 100.0; // Hz
        let mut demod = FmDemodulator::new(200.0, sample_rate);

        // Generate a tone at freq_offset
        let n = 500;
        let input: Vec<Complex<f32>> = (0..n)
            .map(|i| {
                let phase = 2.0 * PI * freq_offset * i as f32 / sample_rate;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect();

        let output = demod.process(&input);

        // After the first sample (startup transient), output should be roughly constant
        let steady = &output[10..];
        let mean: f32 = steady.iter().sum::<f32>() / steady.len() as f32;
        for &val in steady {
            assert!(
                (val - mean).abs() < 0.1,
                "expected steady output, got {val} vs mean {mean}"
            );
        }
    }
}
