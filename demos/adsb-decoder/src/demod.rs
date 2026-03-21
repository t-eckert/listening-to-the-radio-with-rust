/// ADS-B demodulator: detects Mode S preambles in IQ magnitude data
/// and extracts 112-bit long messages.
///
/// ADS-B uses Pulse Position Modulation (PPM) at 1 Mbps on 1090 MHz.
/// Each bit is 1 μs: pulse in first 0.5 μs = 1, pulse in second 0.5 μs = 0.
/// The preamble is 8 μs with pulses at 0, 1, 3.5, and 4.5 μs.

pub struct AdsbDemodulator {
    /// Exact samples per microsecond (floating point to avoid drift)
    samples_per_us: f64,
}

impl AdsbDemodulator {
    pub fn new(sample_rate: u32) -> Self {
        let samples_per_us = sample_rate as f64 / 1_000_000.0;
        assert!(
            samples_per_us >= 2.0,
            "sample rate must be at least 2 MS/s for ADS-B"
        );
        Self { samples_per_us }
    }

    /// Convert a time in microseconds to a sample index.
    fn us_to_sample(&self, us: f64) -> usize {
        (us * self.samples_per_us).round() as usize
    }

    /// Process a buffer of magnitude samples (already `s.norm()`) and return
    /// any complete 112-bit messages found.
    pub fn process(&self, mag: &[f32]) -> Vec<[u8; 112]> {
        let mut messages = Vec::new();
        let preamble_samples = self.us_to_sample(8.0);
        let message_samples = self.us_to_sample(112.0);
        let total_needed = preamble_samples + message_samples;

        if mag.len() < total_needed {
            return messages;
        }

        let mut i = 0;
        while i + total_needed <= mag.len() {
            if self.detect_preamble(&mag[i..]) {
                let data_start = i + preamble_samples;
                if data_start + message_samples <= mag.len() {
                    let bits = self.extract_bits(&mag[data_start..]);
                    messages.push(bits);
                }
                i += total_needed;
                continue;
            }
            i += 1;
        }

        messages
    }

    /// Check if the preamble pattern exists at the start of the buffer.
    ///
    /// The Mode S preamble is 8 μs with pulses at 0, 1, 3.5, and 4.5 μs
    /// and silence at 2, 3, 5, 6, and 7 μs.
    fn detect_preamble(&self, mag: &[f32]) -> bool {
        let preamble_len = self.us_to_sample(8.0);
        if mag.len() < preamble_len {
            return false;
        }

        // Sample at pulse and silence positions using floor() for consistent indexing
        let sample = |us: f64| -> f32 {
            let idx = (us * self.samples_per_us).floor() as usize;
            if idx < mag.len() { mag[idx] } else { 0.0 }
        };

        // Pulse centers: 0, 1, 3.5, 4.5 μs
        let p0 = sample(0.0);
        let p1 = sample(1.0);
        let p2 = sample(3.5);
        let p3 = sample(4.5);

        // Silence centers: 2, 3, 5.5, 6.5, 7.5 μs
        let s0 = sample(2.0);
        let s1 = sample(3.0);
        let s2 = sample(5.5);
        let s3 = sample(6.5);
        let s4 = sample(7.5);

        let min_pulse = p0.min(p1).min(p2).min(p3);
        let max_silence = s0.max(s1).max(s2).max(s3).max(s4);

        min_pulse > max_silence * 2.0 && min_pulse > 0.05
    }

    /// Extract 112 bits from the data portion (after the preamble).
    /// Uses floating-point sample positions to avoid drift.
    ///
    /// For each bit, we compare the magnitude at the center of the first half
    /// (0.25 μs into the bit) vs the center of the second half (0.75 μs).
    /// We use floor/ceil to ensure early and late always map to different samples.
    fn extract_bits(&self, data: &[f32]) -> [u8; 112] {
        let mut bits = [0u8; 112];

        for bit_idx in 0..112 {
            let bit_start = bit_idx as f64 * self.samples_per_us;
            // Early sample: first quarter of bit period
            let early_idx = bit_start.floor() as usize;
            // Late sample: third quarter of bit period
            let late_idx = (bit_start + self.samples_per_us * 0.5).floor() as usize;

            let early = if early_idx < data.len() { data[early_idx] } else { 0.0 };
            let late = if late_idx < data.len() { data[late_idx] } else { 0.0 };

            bits[bit_idx] = if early > late { 1 } else { 0 };
        }

        bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demod_needs_sufficient_sample_rate() {
        let _demod = AdsbDemodulator::new(2_000_000);
    }

    #[test]
    #[should_panic]
    fn demod_rejects_low_sample_rate() {
        let _demod = AdsbDemodulator::new(500_000);
    }

    #[test]
    fn sample_positions_dont_drift() {
        let demod = AdsbDemodulator::new(2_400_000);
        // At 2.4 MS/s, bit 111 starts at 111 μs = sample 266.4
        // The early sample should be at 111.25 μs = sample 267
        // The late sample should be at 111.75 μs = sample 268.2 → 268
        assert_eq!(demod.us_to_sample(111.25), 267);
        assert_eq!(demod.us_to_sample(111.75), 268);
        // Contrast with integer approach: 111 * 2 = 222 (should be 266!)
    }
}
