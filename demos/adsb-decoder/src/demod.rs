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
                // Measure bit positions from the frame start, not from a
                // separately rounded data_start. Rounding the preamble length
                // to a whole sample first (8 μs is 19.2 samples at 2.4 MS/s,
                // not 19) shifts every later probe and can push a bit's "late"
                // probe back inside its own pulse.
                let bits = self.extract_bits(&mag[i..]);
                messages.push(bits);
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

        // Sample by rounding to the nearest sample, so a probe lands as close
        // as possible to the time we asked for. Truncating with floor() pulls
        // every probe backwards in time, which at 2.4 MS/s drops three of the
        // four pulse probes into the silence gap that precedes their pulse.
        let sample = |us: f64| -> f32 {
            let idx = self.us_to_sample(us);
            if idx < mag.len() { mag[idx] } else { 0.0 }
        };

        // Each pulse is 0.5 μs wide and starts at 0, 1.0, 3.5 and 4.5 μs, so
        // probe the middle of each pulse rather than its leading edge.
        let p0 = sample(0.25);
        let p1 = sample(1.25);
        let p2 = sample(3.75);
        let p3 = sample(4.75);

        // Middles of the silence gaps: [0.5,1.0], [1.5,3.5] and [5.0,8.0] μs.
        let s0 = sample(0.75);
        let s1 = sample(2.5);
        let s2 = sample(5.5);
        let s3 = sample(6.5);
        let s4 = sample(7.5);

        let min_pulse = p0.min(p1).min(p2).min(p3);
        let max_silence = s0.max(s1).max(s2).max(s3).max(s4);

        min_pulse > max_silence * 2.0 && min_pulse > 0.05
    }

    /// Extract 112 bits from a frame, measured from the start of its preamble.
    ///
    /// Data begins 8 μs in and each bit lasts 1 μs. For each bit we compare the
    /// magnitude at the middle of the first half (0.25 μs into the bit) with the
    /// middle of the second half (0.75 μs): a pulse in the first half is a 1.
    /// Every position is computed from the frame start so rounding error cannot
    /// accumulate across the 112 bits.
    fn extract_bits(&self, frame: &[f32]) -> [u8; 112] {
        let mut bits = [0u8; 112];

        for bit_idx in 0..112 {
            let bit_start_us = 8.0 + bit_idx as f64;
            let early_idx = self.us_to_sample(bit_start_us + 0.25);
            let late_idx = self.us_to_sample(bit_start_us + 0.75);

            let early = if early_idx < frame.len() { frame[early_idx] } else { 0.0 };
            let late = if late_idx < frame.len() { frame[late_idx] } else { 0.0 };

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

    /// Render a Mode S frame (8 μs preamble + 112 data bits) into a magnitude
    /// buffer, working in continuous time so it is independent of the sample
    /// positions the demodulator happens to pick.
    fn synth(bits: &[u8; 112], sample_rate: u32, lead_us: f64) -> Vec<f32> {
        let spu = sample_rate as f64 / 1_000_000.0;
        // Preamble pulses, each 0.5 μs wide, at 0, 1.0, 3.5, 4.5 μs.
        let mut pulses: Vec<(f64, f64)> = vec![(0.0, 0.5), (1.0, 1.5), (3.5, 4.0), (4.5, 5.0)];
        // PPM data: bit 1 = pulse in the first half of its μs, bit 0 = second half.
        for (i, &b) in bits.iter().enumerate() {
            let start = 8.0 + i as f64 + if b == 1 { 0.0 } else { 0.5 };
            pulses.push((start, start + 0.5));
        }
        let total_us = lead_us + 8.0 + 112.0 + 10.0;
        let n = (total_us * spu).ceil() as usize;
        (0..n)
            .map(|k| {
                let t = k as f64 / spu - lead_us;
                if pulses.iter().any(|&(a, b)| t >= a && t < b) {
                    1.0
                } else {
                    0.01
                }
            })
            .collect()
    }

    #[test]
    fn decodes_a_clean_synthetic_frame() {
        // A DF17 frame: first five bits are 10001.
        let mut bits = [0u8; 112];
        for (i, b) in [1, 0, 0, 0, 1].iter().enumerate() {
            bits[i] = *b;
        }
        for i in 5..112 {
            bits[i] = ((i * 7 + 3) % 5 % 2) as u8;
        }

        for rate in [2_000_000u32, 2_400_000] {
            let demod = AdsbDemodulator::new(rate);
            let mag = synth(&bits, rate, 3.0);
            let found = demod.process(&mag);
            assert_eq!(
                found.len(),
                1,
                "expected exactly one frame at {rate} S/s, got {}",
                found.len()
            );
            assert_eq!(
                found[0].as_slice(),
                bits.as_slice(),
                "bits did not round-trip at {rate} S/s"
            );
        }
    }

    #[test]
    fn preamble_probes_land_inside_the_pulses() {
        // At 2.4 MS/s the four preamble pulses occupy [0,0.5], [1,1.5],
        // [3.5,4], [4.5,5] μs. A probe for each must fall inside its pulse,
        // not in the silence before it.
        let demod = AdsbDemodulator::new(2_400_000);
        let spu = 2.4;
        for (pulse_start, pulse_end) in [(0.0, 0.5), (1.0, 1.5), (3.5, 4.0), (4.5, 5.0)] {
            let idx = demod.us_to_sample(pulse_start + 0.25);
            let t = idx as f64 / spu;
            assert!(
                t >= pulse_start && t < pulse_end,
                "probe for pulse [{pulse_start},{pulse_end}) sampled t={t:.4} μs, outside the pulse"
            );
        }
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
