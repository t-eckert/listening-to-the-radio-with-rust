use num_complex::Complex;
use rustfft::{FftPlanner, num_complex::Complex as FftComplex};
use sdr::dsp::power;

pub struct ScanEntry {
    pub freq_hz: u64,
    pub power_db: f32,
}

#[allow(dead_code)]
pub struct FrequencyScanner {
    start_hz: u64,
    end_hz: u64,
    step_hz: u32,
    sample_rate: u32,
    threshold_db: f32,
    current_index: usize,
    num_steps: usize,
    /// Power level at each frequency step
    spectrum: Vec<f32>,
    /// Detected signals above threshold
    signals: Vec<ScanEntry>,
    /// Number of complete sweeps
    sweep_count: u32,
    /// Estimated noise floor (median power)
    noise_floor: f32,
}

impl FrequencyScanner {
    pub fn new(
        start_hz: u64,
        end_hz: u64,
        step_hz: u32,
        sample_rate: u32,
        threshold_db: f32,
    ) -> Self {
        let num_steps = ((end_hz - start_hz) / step_hz as u64) as usize + 1;
        Self {
            start_hz,
            end_hz,
            step_hz,
            sample_rate,
            threshold_db,
            current_index: 0,
            num_steps,
            spectrum: vec![-120.0; num_steps],
            signals: Vec::new(),
            sweep_count: 0,
            noise_floor: -120.0,
        }
    }

    pub fn current_freq(&self) -> u64 {
        self.start_hz + self.current_index as u64 * self.step_hz as u64
    }

    pub fn measure(&mut self, samples: &[Complex<f32>]) {
        // Compute average power using FFT for better spectral estimate
        let n = samples.len().min(4096).next_power_of_two();
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(n);

        // Apply window and convert to rustfft Complex type
        let mut fft_buf: Vec<FftComplex<f32>> = samples[..n]
            .iter()
            .enumerate()
            .map(|(i, s)| {
                // Hamming window
                let w =
                    0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32).cos();
                FftComplex::new(s.re * w, s.im * w)
            })
            .collect();

        fft.process(&mut fft_buf);

        // The signal of interest is centered at DC (bin 0) since we tune the SDR
        // to each frequency. After FFT, bins near 0 and near N contain the signal.
        // Skip bin 0 itself to avoid the RTL-SDR DC spike (LO leakage).
        //
        // Use the inner 50% of bandwidth: bins 1..N/4 and bins 3N/4..N-1
        let quarter = n / 4;
        let low_bins = &fft_buf[1..quarter];
        let high_bins = &fft_buf[n - quarter..];
        let total_bins = low_bins.len() + high_bins.len();

        let sum_power: f32 = low_bins
            .iter()
            .chain(high_bins.iter())
            .map(|c| c.re * c.re + c.im * c.im)
            .sum::<f32>();

        let avg_power = sum_power / total_bins as f32 / (n as f32 * n as f32);
        let db = power::to_db(avg_power);
        self.spectrum[self.current_index] = db;
    }

    /// Advance to the next frequency step. Returns true if a sweep just completed.
    pub fn step(&mut self) -> bool {
        self.current_index += 1;
        if self.current_index >= self.num_steps {
            self.update_signals();
            self.current_index = 0;
            self.sweep_count += 1;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.spectrum.fill(-120.0);
        self.signals.clear();
        self.sweep_count = 0;
        self.noise_floor = -120.0;
    }

    fn update_signals(&mut self) {
        self.signals.clear();

        // Compute the noise floor as the median power level
        let mut sorted_powers: Vec<f32> = self.spectrum.clone();
        sorted_powers.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let noise_floor = sorted_powers[sorted_powers.len() / 2];
        self.noise_floor = noise_floor;

        let detection_level = noise_floor + self.threshold_db;

        // Find peaks: groups of adjacent bins above the threshold, report only the peak
        let mut i = 0;
        while i < self.spectrum.len() {
            if self.spectrum[i] > detection_level {
                // Found the start of a signal — scan to find the peak
                let mut peak_idx = i;
                let mut peak_db = self.spectrum[i];
                while i < self.spectrum.len() && self.spectrum[i] > detection_level {
                    if self.spectrum[i] > peak_db {
                        peak_db = self.spectrum[i];
                        peak_idx = i;
                    }
                    i += 1;
                }
                self.signals.push(ScanEntry {
                    freq_hz: self.start_hz + peak_idx as u64 * self.step_hz as u64,
                    power_db: peak_db,
                });
            } else {
                i += 1;
            }
        }

        // Sort by frequency
        self.signals
            .sort_by(|a, b| a.freq_hz.cmp(&b.freq_hz));
    }

    // Accessors for the UI
    pub fn spectrum(&self) -> &[f32] {
        &self.spectrum
    }

    pub fn signals(&self) -> &[ScanEntry] {
        &self.signals
    }

    pub fn start_hz(&self) -> u64 {
        self.start_hz
    }

    pub fn end_hz(&self) -> u64 {
        self.end_hz
    }

    #[allow(dead_code)]
    pub fn step_hz(&self) -> u32 {
        self.step_hz
    }

    pub fn sweep_count(&self) -> u32 {
        self.sweep_count
    }

    pub fn progress(&self) -> f64 {
        self.current_index as f64 / self.num_steps as f64
    }

    pub fn threshold_db(&self) -> f32 {
        self.threshold_db
    }

    pub fn noise_floor(&self) -> f32 {
        self.noise_floor
    }
}
