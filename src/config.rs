use std::path::PathBuf;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalKind {
    Constant { voltage: f64 },
    Ramp { start: f64, end: f64 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseKind {
    None,
    Uniform,
    Gaussian,
}
#[derive(Debug, Clone, Copy)]
pub struct NoiseConfig {
    pub kind: NoiseKind,
    pub amplitude_lsb: f64,
}
#[derive(Debug, Clone)]
pub struct ExperimentConfig {
    pub bits: u8,
    pub v_ref: f64,
    pub sample_count: usize,
    pub seed: u64,
    pub signal: SignalKind,
    pub noise: NoiseConfig,
    pub oversampling: Vec<usize>,
    pub histogram_bins: usize,
    pub output_dir: PathBuf,
}
impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            bits: 12,
            v_ref: 3.3,
            sample_count: 1_048_576,
            seed: 42,
            signal: SignalKind::Constant { voltage: 1.234567 },
            noise: NoiseConfig {
                kind: NoiseKind::Gaussian,
                amplitude_lsb: 0.75,
            },
            oversampling: vec![1, 4, 16, 64, 256],
            histogram_bins: 80,
            output_dir: "results".into(),
        }
    }
}
impl ExperimentConfig {
    pub fn validated(&self) -> Result<Self, String> {
        if !(2..=24).contains(&self.bits) {
            return Err(format!("--bits must be in 2..=24, got {}", self.bits));
        }
        if !self.v_ref.is_finite() || self.v_ref <= 0.0 {
            return Err(format!(
                "--vref must be finite and positive, got {}",
                self.v_ref
            ));
        }
        if self.sample_count == 0 {
            return Err(format!(
                "--samples must be positive, got {}",
                self.sample_count
            ));
        }
        if !self.noise.amplitude_lsb.is_finite() || self.noise.amplitude_lsb < 0.0 {
            return Err(format!(
                "--noise-lsb must be finite and non-negative, got {}",
                self.noise.amplitude_lsb
            ));
        }
        let finite = match self.signal {
            SignalKind::Constant { voltage } => voltage.is_finite(),
            SignalKind::Ramp { start, end } => {
                start.is_finite() && end.is_finite() && (end - start).is_finite()
            }
        };
        if !finite {
            return Err("signal voltages and ramp span must be finite".into());
        }
        if self.histogram_bins < 2 {
            return Err(format!(
                "--histogram-bins must be at least 2, got {}",
                self.histogram_bins
            ));
        }
        if self.oversampling.is_empty() {
            return Err("--osr must not be empty".into());
        }
        for &m in &self.oversampling {
            if !m.is_power_of_two() || m > self.sample_count || !self.sample_count.is_multiple_of(m)
            {
                return Err(format!(
                    "--osr {m} must be a power of two dividing --samples"
                ));
            }
        }
        let mut config = self.clone();
        config.oversampling.push(1);
        config.oversampling.sort_unstable();
        config.oversampling.dedup();
        Ok(config)
    }
}
#[derive(Debug, Clone)]
pub struct SweepConfig {
    pub base: ExperimentConfig,
    pub amplitudes_lsb: Vec<f64>,
}
impl Default for SweepConfig {
    fn default() -> Self {
        Self {
            base: ExperimentConfig::default(),
            amplitudes_lsb: vec![0.0, 0.05, 0.10, 0.25, 0.50, 0.75, 1.0, 2.0],
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validation() {
        let mut c = ExperimentConfig {
            oversampling: vec![16, 4, 16],
            ..Default::default()
        };
        assert_eq!(c.validated().unwrap().oversampling, [1, 4, 16]);
        c.bits = 1;
        assert!(c.validated().is_err());
        c.bits = 12;
        c.v_ref = f64::NAN;
        assert!(c.validated().is_err());
        c.v_ref = 3.3;
        c.sample_count = 3;
        assert!(c.validated().is_err());
    }
}
