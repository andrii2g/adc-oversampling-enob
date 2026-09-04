use crate::{
    config::{NoiseConfig, NoiseKind},
    rng::SplitMix64,
};
pub struct NoiseGenerator {
    config: NoiseConfig,
    lsb_volts: f64,
    rng: SplitMix64,
    gaussian_spare: Option<f64>,
}
impl NoiseGenerator {
    pub fn new(config: NoiseConfig, lsb_volts: f64, seed: u64) -> Self {
        Self {
            config,
            lsb_volts,
            rng: SplitMix64::new(seed),
            gaussian_spare: None,
        }
    }
    pub fn sample_volts(&mut self) -> f64 {
        let unit = match self.config.kind {
            NoiseKind::None => return 0.0,
            NoiseKind::Uniform => 2.0 * self.rng.next_f64() - 1.0,
            NoiseKind::Gaussian => match self.gaussian_spare.take() {
                Some(value) => value,
                None => {
                    let radius = (-2.0 * (1.0 - self.rng.next_f64()).ln()).sqrt();
                    let angle = std::f64::consts::TAU * self.rng.next_f64();
                    self.gaussian_spare = Some(radius * angle.sin());
                    radius * angle.cos()
                }
            },
        };
        unit * self.config.amplitude_lsb * self.lsb_volts
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn noise_properties() {
        for kind in [NoiseKind::None, NoiseKind::Uniform, NoiseKind::Gaussian] {
            let config = NoiseConfig {
                kind,
                amplitude_lsb: 0.75,
            };
            let mut a = NoiseGenerator::new(config, 1.0, 42);
            let mut b = NoiseGenerator::new(config, 1.0, 42);
            let mut sum = 0.0;
            let mut squares = 0.0;
            for _ in 0..100000 {
                let x = a.sample_volts();
                assert_eq!(x, b.sample_volts());
                if kind == NoiseKind::None {
                    assert_eq!(x, 0.0);
                }
                if kind == NoiseKind::Uniform {
                    assert!(x.abs() <= 0.75);
                }
                sum += x;
                squares += x * x;
            }
            assert!((sum / 100000.0).abs() < 0.01);
            let expected = match kind {
                NoiseKind::None => 0.0,
                NoiseKind::Uniform => 0.75 / 3.0_f64.sqrt(),
                NoiseKind::Gaussian => 0.75,
            };
            assert!(((squares / 100000.0).sqrt() - expected).abs() < 0.01);
        }
    }
}
