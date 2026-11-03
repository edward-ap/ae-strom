use super::builtin_presets::EQ_FREQUENCIES;
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type as BiquadType};
use num_complex::Complex32;
use realfft::RealFftPlanner;
use std::f32::consts::PI;

const EQ_Q: f32 = 0.9;
const EQ_SMOOTH_ALPHA: f32 = 0.25;

/// Equalizer configuration: preamp plus 10 band gains.
///
/// - `preamp`: input gain applied before per-band filters.
/// - `bands`: array of 10 band gains (dB) from low to high frequencies.
#[derive(Debug, Clone)]
pub struct EqSettings {
    pub preamp: f32,
    pub bands: [f32; 10],
}

impl Default for EqSettings {
    fn default() -> Self {
        Self {
            preamp: 0.0,
            bands: [0.0; 10],
        }
    }
}

/// Runtime EQ state that smooths transitions and holds per-channel filter state.
#[derive(Debug, Clone)]
pub struct EqRuntimeState {
    pub current: EqSettings,
    target: EqSettings,
    coeffs: Option<[Coefficients<f32>; 10]>,
    states: Vec<Vec<DirectForm1<f32>>>,
    preamp_gain: f32,
}

impl EqRuntimeState {
    /// Create runtime EQ state for the given settings, channel count, and sample rate.
    pub fn new(settings: EqSettings, channels: usize, sample_rate: f32) -> Self {
        let coeffs = Some(build_eq_coeffs(&settings, sample_rate));
        let states = coeffs
            .as_ref()
            .map(|c| init_states(channels, c))
            .unwrap_or_default();
        let preamp_gain = db_to_amp(settings.preamp);

        Self {
            current: settings.clone(),
            target: settings,
            coeffs,
            states,
            preamp_gain,
        }
    }

    /// Update the target EQ curve to which the runtime state will smooth.
    pub fn update_target(&mut self, target: &EqSettings) {
        self.target = target.clone();
    }

    #[allow(clippy::needless_range_loop)]
    /// Smooth current settings toward the target and refresh filter coefficients.
    pub fn smooth(&mut self, sample_rate: f32) {
        self.current.preamp += (self.target.preamp - self.current.preamp) * EQ_SMOOTH_ALPHA;
        for b in 0..10 {
            self.current.bands[b] +=
                (self.target.bands[b] - self.current.bands[b]) * EQ_SMOOTH_ALPHA;
        }
        self.preamp_gain = db_to_amp(self.current.preamp);
        self.coeffs = Some(build_eq_coeffs(&self.current, sample_rate));
        if let Some(coeffs) = &self.coeffs {
            if self.states.is_empty() {
                self.states = init_states(self.states.len().max(1), coeffs);
            }
            for ch in 0..self.states.len() {
                for band in 0..10 {
                    self.states[ch][band].update_coefficients(coeffs[band]);
                }
            }
        }
    }

    /// Apply EQ to an interleaved buffer in-place.
    pub fn apply_eq(&mut self, buf: &mut [f32], channels: usize) {
        if channels == 0 || buf.is_empty() {
            return;
        }
        if self.states.len() < channels {
            if let Some(coeffs) = &self.coeffs {
                self.states = init_states(channels, coeffs);
            }
        }

        for frame in buf.chunks_exact_mut(channels) {
            for (ch_idx, sample) in frame.iter_mut().enumerate() {
                let mut s = *sample * self.preamp_gain;
                if ch_idx < self.states.len() {
                    for band in 0..10 {
                        s = self.states[ch_idx][band].run(s);
                    }
                }
                *sample = s;
            }
        }
    }
}

/// Apply volume gain to an audio buffer in-place.
///
/// Multiplies each sample by the specified gain value. Typically used
/// to apply master volume and mute state.
pub fn apply_volume(buf: &mut [f32], volume_gain: f32) {
    for s in buf {
        *s *= volume_gain;
    }
}

/// Simple FFT-based spectrum analyzer used for UI visuals.
pub struct SpectrumAnalyzer {
    sample_rate: u32,
    fft_size: usize,
    window: Vec<f32>,
    buffer: Vec<f32>,
    write_pos: usize,
    smoothed: [f32; 5],
    plan: std::sync::Arc<dyn realfft::RealToComplex<f32>>,
    scratch: Vec<Complex32>,
    spectrum: Vec<Complex32>,
}

impl SpectrumAnalyzer {
    /// Create a new spectrum analyzer with the given sample rate.
    ///
    /// Initializes a 1024-point FFT with Hann windowing for frequency analysis.
    pub fn new(sample_rate: u32) -> Self {
        let fft_size = 1024usize;
        let mut planner = RealFftPlanner::<f32>::new();
        let window: Vec<f32> = (0..fft_size)
            .map(|n| 0.5 * (1.0 - (2.0 * PI * n as f32 / (fft_size as f32 - 1.0)).cos()))
            .collect();
        let plan = planner.plan_fft_forward(fft_size);
        let scratch = plan.make_scratch_vec();
        let spectrum = plan.make_output_vec();

        Self {
            sample_rate,
            fft_size,
            window,
            buffer: vec![0.0; fft_size],
            write_pos: 0,
            smoothed: [0.0; 5],
            plan,
            scratch,
            spectrum,
        }
    }

    /// Push audio samples into the analyzer's circular buffer.
    ///
    /// When the buffer fills to FFT size, automatically triggers frequency analysis
    /// and updates the smoothed spectrum levels.
    pub fn push_block(&mut self, samples: &[f32]) {
        for &s in samples {
            self.buffer[self.write_pos] = s;
            self.write_pos += 1;
            if self.write_pos >= self.fft_size {
                self.analyze_block();
                self.write_pos = 0;
            }
        }
    }

    /// Get the current smoothed spectrum levels for 5 frequency bands.
    ///
    /// Returns values normalized to [0.0, 1.0] range for UI visualization.
    pub fn levels(&self) -> [f32; 5] {
        self.smoothed
    }

    /// Perform FFT analysis on the accumulated buffer and update smoothed levels.
    ///
    /// Applies Hann windowing, calculates magnitudes per frequency band,
    /// normalizes relative to peak, and smooths with attack/release characteristics.
    fn analyze_block(&mut self) {
        let mut input: Vec<f32> = self
            .buffer
            .iter()
            .zip(self.window.iter())
            .map(|(s, w)| s * w)
            .collect();

        self.plan
            .process_with_scratch(&mut input, &mut self.spectrum, &mut self.scratch)
            .ok();

        let mut band_peak = [0.0f32; 5];
        let mut global_peak = 1e-6f32;

        for (i, c) in self.spectrum.iter().enumerate() {
            if i == 0 {
                continue;
            }
            let freq = (i as f32 * self.sample_rate as f32) / self.fft_size as f32;
            let band_idx = band_for_freq(freq);
            let mag = (c.re * c.re + c.im * c.im).sqrt() / self.fft_size as f32;
            band_peak[band_idx] = band_peak[band_idx].max(mag);
            if mag > global_peak {
                global_peak = mag;
            }
        }

        const BAND_GAIN: [f32; 5] = [0.35, 0.75, 0.9, 1.0, 1.15];
        for b in 0..5 {
            let mut rel = (band_peak[b] / global_peak) * BAND_GAIN[b];
            rel = rel.clamp(0.0, 1.5);
            let db = 20.0 * (rel + 1e-6).log10(); // relative to frame peak
            let target = ((db + 40.0) / 40.0).clamp(0.0, 1.0);
            let current = self.smoothed[b];
            let attack = 0.65;
            let release = 0.25;
            self.smoothed[b] = if target > current {
                current + (target - current) * attack
            } else {
                current + (target - current) * release
            };
        }
    }
}

/// Map a frequency to one of 5 spectrum bands for visualization.
///
/// Bands: 0 = <250Hz, 1 = 250-1000Hz, 2 = 1-4kHz, 3 = 4-8kHz, 4 = >8kHz
fn band_for_freq(freq: f32) -> usize {
    if freq < 250.0 {
        0
    } else if freq < 1000.0 {
        1
    } else if freq < 4000.0 {
        2
    } else if freq < 8000.0 {
        3
    } else {
        4
    }
}

/// Convert decibels to linear amplitude gain.
fn db_to_amp(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

/// Build biquad filter coefficients for all 10 EQ bands.
///
/// Creates peaking EQ filters at standard frequencies with gains from settings.
/// Handles edge cases where frequencies exceed Nyquist limit.
fn build_eq_coeffs(settings: &EqSettings, fs: f32) -> [Coefficients<f32>; 10] {
    let mut coeffs: [Coefficients<f32>; 10] = std::array::from_fn(|_| {
        Coefficients::<f32>::from_params(BiquadType::PeakingEQ(0.0), fs.hz(), 1000.0.hz(), EQ_Q)
            .expect("default coeffs")
    });

    for (idx, c) in coeffs.iter_mut().enumerate() {
        let f0 = (EQ_FREQUENCIES[idx] as f32).min(fs * 0.45);
        let gain_db = settings.bands[idx];
        *c = Coefficients::<f32>::from_params(
            BiquadType::PeakingEQ(gain_db),
            fs.hz(),
            f0.hz(),
            EQ_Q,
        )
        .unwrap_or_else(|_| {
            let safe_f0 = f0.max(10.0);
            Coefficients::<f32>::from_params(
                BiquadType::PeakingEQ(gain_db),
                fs.hz(),
                safe_f0.hz(),
                EQ_Q,
            )
            .expect("fallback coeffs")
        });
    }

    coeffs
}

/// Initialize biquad filter states for all channels and EQ bands.
///
/// Creates a 2D structure: [channel][band] of DirectForm1 filters,
/// each initialized with the corresponding coefficients.
fn init_states(channels: usize, coeffs: &[Coefficients<f32>; 10]) -> Vec<Vec<DirectForm1<f32>>> {
    (0..channels)
        .map(|_| {
            (0..10)
                .map(|b| DirectForm1::<f32>::new(coeffs[b]))
                .collect()
        })
        .collect()
}
