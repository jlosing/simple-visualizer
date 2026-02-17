use once_cell::sync::Lazy;
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::Arc;

/// Constants
const SAMPLE_RATE: f32 = 44100.0;
const N: usize = 4096;
const BANDS: usize = 16;
const F_MIN: f32 = 20.0;
// High bands didnt carry much information in them
const F_MAX: f32 = 14000.0;

static FFT_PLAN: Lazy<Arc<dyn rustfft::Fft<f32>>> = Lazy::new(|| {
    let mut planner = FftPlanner::<f32>::new();
    planner.plan_fft_forward(N)
});

static BAND_BINS: Lazy<Vec<(usize, usize)>> = Lazy::new(|| {
    let mut bins = Vec::with_capacity(BANDS);
    let mut current_bin = 1; // Start at 1 to skip DC offset

    for i in 0..BANDS {
        let f_end = F_MIN * (F_MAX / F_MIN).powf((i + 1) as f32 / BANDS as f32);

        let target_end_bin = ((f_end / SAMPLE_RATE) * N as f32).round() as usize;

        let start_bin = current_bin;
        let mut end_bin = target_end_bin;

        if end_bin <= start_bin {
            end_bin = start_bin + 1;
        }

        bins.push((start_bin, end_bin));
        current_bin = end_bin; // Next band starts EXACTLY where this one ends
    }
    bins
});

static mut BUFFER: Lazy<Vec<Complex<f32>>> = Lazy::new(|| vec![Complex { re: 0.0, im: 0.0 }; N]);
static mut MAGNITUDES: Lazy<Vec<f32>> = Lazy::new(|| vec![0.0; N / 2]);

const SAMPLE_LEN: usize = 1024; // Your audio chunk size
const SCALING_FACTOR: f32 = 2.0 / (SAMPLE_LEN as f32);
const FFT_LEN: usize = 4096;

pub fn fft_calc(samples: Vec<f32>) -> Vec<u64> {
    // Unsafe doesn't seem the best here. I will look into alternatives
    unsafe {
        for i in 0..FFT_LEN {
            if i < SAMPLE_LEN {
                // Apply Hann Window scaled to the AUDIO length (1024), NOT the FFT length
                let t = i as f32 / (SAMPLE_LEN as f32 - 1.0);
                let hann_window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * t).cos());

                BUFFER[i].re = samples[i] * hann_window;
            } else {
                BUFFER[i].re = 0.0;
            }
            BUFFER[i].im = 0.0;
        }

        FFT_PLAN.process(&mut BUFFER);

        for i in 0..(N / 2) {
            MAGNITUDES[i] = BUFFER[i].norm();
        }

        let mut band_vals = vec![0; BANDS];

        for (i, &(start, end)) in BAND_BINS.iter().enumerate() {
            let mut peak_magnitude = 0.0_f32;
            let safe_end = end.min(N / 2);

            for bin in start..safe_end {
                if MAGNITUDES[bin] > peak_magnitude {
                    peak_magnitude = MAGNITUDES[bin]
                }
            }

            let scaled_peak = peak_magnitude * SCALING_FACTOR;

            let db = 20.0 * (scaled_peak + 1e-7).log10();

            let tilt = i as f32 * 1.2;
            let tilted_db = db + tilt;

            let min_db = -60.0;
            let max_db = 0.0;
            let normalized = (tilted_db - min_db) / (max_db - min_db);

            band_vals[i] = (normalized.clamp(0.0, 1.0) * 100.0) as u64;
        }

        band_vals
    }
}
