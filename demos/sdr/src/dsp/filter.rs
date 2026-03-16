use num_complex::Complex;
use std::f32::consts::PI;

use super::window;

/// FIR low-pass filter with decimation.
pub struct LowPassFilter {
    taps: Vec<f32>,
    history: Vec<Complex<f32>>,
    decimation: usize,
    pos: usize,
    decim_counter: usize,
}

impl LowPassFilter {
    /// Create a new low-pass filter.
    /// - `cutoff`: cutoff frequency as a fraction of the sample rate (0.0 to 0.5)
    /// - `num_taps`: number of filter taps (odd for symmetric)
    /// - `decimation`: keep every Nth output sample
    pub fn new(cutoff: f32, num_taps: usize, decimation: usize) -> Self {
        let taps = design_lowpass(cutoff, num_taps);
        Self {
            history: vec![Complex::new(0.0, 0.0); taps.len()],
            taps,
            decimation,
            pos: 0,
            decim_counter: 0,
        }
    }

    /// Filter and decimate complex IQ samples.
    pub fn process(&mut self, input: &[Complex<f32>]) -> Vec<Complex<f32>> {
        let mut output = Vec::with_capacity(input.len() / self.decimation + 1);

        for &sample in input {
            self.history[self.pos] = sample;
            self.pos = (self.pos + 1) % self.history.len();
            self.decim_counter += 1;

            if self.decim_counter >= self.decimation {
                self.decim_counter = 0;
                let mut sum = Complex::new(0.0, 0.0);
                for j in 0..self.taps.len() {
                    let idx = (self.pos + j) % self.taps.len();
                    sum += self.history[idx] * self.taps[j];
                }
                output.push(sum);
            }
        }

        output
    }

    /// Filter and decimate real-valued samples.
    pub fn process_real(&mut self, input: &[f32]) -> Vec<f32> {
        let complex_input: Vec<Complex<f32>> =
            input.iter().map(|&x| Complex::new(x, 0.0)).collect();
        self.process(&complex_input).iter().map(|c| c.re).collect()
    }
}

/// Design a Hamming-windowed sinc low-pass filter.
fn design_lowpass(cutoff: f32, num_taps: usize) -> Vec<f32> {
    let win = window::hamming(num_taps);
    let center = (num_taps - 1) as f32 / 2.0;

    let mut taps: Vec<f32> = (0..num_taps)
        .map(|i| {
            let x = i as f32 - center;
            let sinc = if x.abs() < 1e-6 {
                2.0 * cutoff
            } else {
                (2.0 * PI * cutoff * x).sin() / (PI * x)
            };
            sinc * win[i]
        })
        .collect();

    // Normalize so taps sum to 1
    let sum: f32 = taps.iter().sum();
    if sum.abs() > 1e-10 {
        for t in &mut taps {
            *t /= sum;
        }
    }

    taps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowpass_dc_passes() {
        let mut filter = LowPassFilter::new(0.25, 31, 1);
        let dc: Vec<Complex<f32>> = vec![Complex::new(1.0, 0.0); 200];
        let out = filter.process(&dc);
        // After settling, output should be close to 1.0
        let last = out.last().unwrap();
        assert!((last.re - 1.0).abs() < 0.05, "DC should pass: got {}", last.re);
    }
}
