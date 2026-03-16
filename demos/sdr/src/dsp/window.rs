use std::f32::consts::PI;

/// Generate Hamming window coefficients of length `n`.
pub fn hamming(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| 0.54 - 0.46 * (2.0 * PI * i as f32 / (n - 1) as f32).cos())
        .collect()
}

/// Generate Blackman window coefficients of length `n`.
pub fn blackman(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let x = 2.0 * PI * i as f32 / (n - 1) as f32;
            0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hamming_endpoints() {
        let w = hamming(64);
        assert!(w[0] < 0.1); // Near zero at edges
        assert!(w[31] > 0.9); // Near one at center
    }

    #[test]
    fn blackman_endpoints() {
        let w = blackman(64);
        assert!(w[0] < 0.01);
        assert!(w[31] > 0.9);
    }
}
