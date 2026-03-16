use num_complex::Complex;

/// Compute the average power of a buffer of IQ samples.
pub fn average_power(samples: &[Complex<f32>]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|s| s.norm_sqr()).sum();
    sum / samples.len() as f32
}

/// Convert a linear power value to decibels.
pub fn to_db(power: f32) -> f32 {
    if power <= 0.0 {
        return -120.0; // noise floor
    }
    10.0 * power.log10()
}

/// Compute the power spectrum in dB from IQ samples (after FFT).
/// `magnitudes` should be the squared magnitudes of the FFT output.
pub fn power_spectrum_db(magnitudes: &[f32]) -> Vec<f32> {
    magnitudes.iter().map(|&m| to_db(m)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_power() {
        let samples = vec![Complex::new(1.0, 0.0); 100];
        let power = average_power(&samples);
        assert!((power - 1.0).abs() < 0.01);
    }

    #[test]
    fn db_conversion() {
        assert!((to_db(1.0) - 0.0).abs() < 0.01);
        assert!((to_db(10.0) - 10.0).abs() < 0.01);
        assert!((to_db(0.1) - -10.0).abs() < 0.01);
    }
}
