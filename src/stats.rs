use crate::oversample::DecimatedSample;
/// Welford accumulation; all variance estimates describe the population (divide by N).
#[derive(Debug, Clone, Copy, Default)]
pub struct RunningStats {
    count: u64,
    mean: f64,
    m2: f64,
}
impl RunningStats {
    pub fn push(&mut self, x: f64) {
        self.count += 1;
        let delta = x - self.mean;
        self.mean += delta / self.count as f64;
        self.m2 += delta * (x - self.mean);
    }
    pub fn count(&self) -> u64 {
        self.count
    }
    pub fn mean(&self) -> Option<f64> {
        (self.count > 0).then_some(self.mean)
    }
    pub fn population_variance(&self) -> Option<f64> {
        (self.count > 0).then(|| (self.m2 / self.count as f64).max(0.0))
    }
    pub fn population_std_dev(&self) -> Option<f64> {
        self.population_variance().map(f64::sqrt)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Metrics {
    pub sample_count: usize,
    pub bias: f64,
    pub std_dev: f64,
    pub rmse: f64,
    pub snr_db: Option<f64>,
    pub snr_enob: Option<f64>,
}
/// Scaled sum of squares avoids overflow from squaring otherwise finite errors.
fn rms(values: impl Iterator<Item = f64>) -> f64 {
    let mut scale = 0.0_f64;
    let mut sum = 0.0;
    let mut count = 0usize;
    for value in values {
        count += 1;
        let a = value.abs();
        if a > 0.0 {
            if a > scale {
                sum = 1.0 + sum * (scale / a).powi(2);
                scale = a;
            } else {
                sum += (a / scale).powi(2);
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        scale * (sum / count as f64).sqrt()
    }
}
pub fn analyze(samples: &[DecimatedSample]) -> Metrics {
    let mut errors = RunningStats::default();
    let mut reference = RunningStats::default();
    for s in samples {
        errors.push(s.measured - s.reference);
        reference.push(s.reference);
    }
    let rmse = rms(samples.iter().map(|s| s.measured - s.reference));
    let reference_mean = reference.mean().unwrap_or(0.0);
    let ac = rms(samples.iter().map(|s| s.reference - reference_mean));
    let magnitude = samples
        .iter()
        .map(|s| s.reference.abs())
        .fold(0.0_f64, f64::max);
    let snr_db =
        if ac > f64::EPSILON * magnitude && rmse > 0.0 && ac.is_finite() && rmse.is_finite() {
            let value = 20.0 * (ac.log10() - rmse.log10());
            value.is_finite().then_some(value)
        } else {
            None
        };
    Metrics {
        sample_count: samples.len(),
        bias: errors.mean().unwrap_or(0.0),
        std_dev: errors.population_std_dev().unwrap_or(0.0),
        rmse,
        snr_db,
        snr_enob: snr_db.map(|x| (x - 1.76) / 6.02),
    }
}
pub fn theoretical_enob_gain(osr: usize) -> f64 {
    0.5 * (osr as f64).log2()
}
pub fn relative_gain(reference_error: f64, current_error: f64) -> Option<f64> {
    if reference_error > 0.0
        && current_error > 0.0
        && reference_error.is_finite()
        && current_error.is_finite()
    {
        Some(reference_error.log2() - current_error.log2())
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn population_and_metrics() {
        let mut s = RunningStats::default();
        assert_eq!(s.mean(), None);
        for x in [1.0, 2.0, 3.0, 4.0] {
            s.push(x);
        }
        assert_eq!(s.count(), 4);
        assert_eq!(s.mean(), Some(2.5));
        assert_eq!(s.population_variance(), Some(1.25));
        let m = analyze(&[
            DecimatedSample {
                reference: 1.0,
                measured: 2.0,
            },
            DecimatedSample {
                reference: 3.0,
                measured: 2.0,
            },
        ]);
        assert_eq!(m.bias, 0.0);
        assert_eq!(m.std_dev, 1.0);
        assert_eq!(m.rmse, 1.0);
        assert_eq!(m.snr_db, Some(0.0));
        let c = analyze(
            &[DecimatedSample {
                reference: 1.0,
                measured: 2.0,
            }; 4],
        );
        assert_eq!(c.snr_db, None);
        assert_eq!(c.rmse, 1.0);
    }
    #[test]
    fn gains() {
        for (m, bits) in [(1, 0.0), (4, 1.0), (16, 2.0), (64, 3.0), (256, 4.0)] {
            assert_eq!(theoretical_enob_gain(m), bits);
        }
        assert_eq!(relative_gain(2.0, 1.0), Some(1.0));
        assert_eq!(relative_gain(0.0, 0.0), None);
        assert_eq!(relative_gain(1.0, 0.0), None);
    }
}
