use crate::{
    adc::Adc,
    config::ExperimentConfig,
    noise::NoiseGenerator,
    oversample, signal,
    stats::{self, Metrics},
};
pub type ExperimentError = String;
#[derive(Debug, Clone)]
pub struct RawDataset {
    pub reference_volts: Vec<f64>,
    pub measured_volts: Vec<f64>,
    pub codes: Vec<u32>,
}
#[derive(Debug, Clone)]
pub struct OversamplingResult {
    pub osr: usize,
    pub output_samples: usize,
    pub metrics: Metrics,
    pub rmse_gain_bits: Option<f64>,
    pub stddev_gain_bits: Option<f64>,
    pub theoretical_gain_bits: f64,
    pub errors_lsb: Vec<f64>,
}
#[derive(Debug, Clone)]
pub struct ExperimentResult {
    pub config: ExperimentConfig,
    pub adc_lsb_volts: f64,
    pub raw_code_min: u32,
    pub raw_code_max: u32,
    pub raw_unique_codes: usize,
    pub results: Vec<OversamplingResult>,
}
pub fn run(config: &ExperimentConfig) -> Result<ExperimentResult, ExperimentError> {
    let config = config.validated()?;
    let adc = Adc::new(config.bits, config.v_ref)?;
    if adc.lsb() == 0.0 || !(config.noise.amplitude_lsb * adc.lsb()).is_finite() {
        return Err("Vref/noise scaling exceeds finite numeric range".into());
    }
    let mut noise = NoiseGenerator::new(config.noise, adc.lsb(), config.seed);
    let mut raw = RawDataset {
        reference_volts: signal::generate(config.signal, config.sample_count),
        measured_volts: Vec::with_capacity(config.sample_count),
        codes: Vec::with_capacity(config.sample_count),
    };
    for &reference in &raw.reference_volts {
        let noisy = reference + noise.sample_volts();
        if !noisy.is_finite() {
            return Err("analog input plus noise exceeds finite numeric range".into());
        }
        let (code, measured) = adc.quantize_and_decode(noisy);
        raw.codes.push(code);
        raw.measured_volts.push(measured);
    }
    let mut results = Vec::new();
    for &osr in &config.oversampling {
        let samples = oversample::decimate(&raw.reference_volts, &raw.measured_volts, osr)?;
        let metrics = stats::analyze(&samples);
        let errors_lsb: Vec<_> = samples
            .iter()
            .map(|s| (s.measured - s.reference) / adc.lsb())
            .collect();
        if !metrics.bias.is_finite()
            || !metrics.std_dev.is_finite()
            || !metrics.rmse.is_finite()
            || errors_lsb.iter().any(|x| !x.is_finite())
        {
            return Err("experiment metrics exceed finite numeric range".into());
        }
        results.push(OversamplingResult {
            osr,
            output_samples: samples.len(),
            metrics,
            rmse_gain_bits: None,
            stddev_gain_bits: None,
            theoretical_gain_bits: stats::theoretical_enob_gain(osr),
            errors_lsb,
        });
    }
    let baseline = results[0].metrics.clone();
    for r in &mut results {
        r.rmse_gain_bits = stats::relative_gain(baseline.rmse, r.metrics.rmse);
        r.stddev_gain_bits = stats::relative_gain(baseline.std_dev, r.metrics.std_dev);
    }
    raw.codes.sort_unstable();
    raw.codes.dedup();
    Ok(ExperimentResult {
        adc_lsb_volts: adc.lsb(),
        raw_code_min: raw.codes[0],
        raw_code_max: *raw.codes.last().unwrap(),
        raw_unique_codes: raw.codes.len(),
        config,
        results,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NoiseKind;
    #[test]
    fn repeatable_and_no_dither() {
        let mut c = ExperimentConfig {
            sample_count: 4096,
            ..Default::default()
        };
        assert_eq!(
            run(&c).unwrap().results[2].metrics,
            run(&c).unwrap().results[2].metrics
        );
        c.noise.kind = NoiseKind::None;
        let r = run(&c).unwrap();
        assert_eq!(r.raw_unique_codes, 1);
        for row in &r.results {
            assert_eq!(row.metrics.rmse, r.results[0].metrics.rmse);
            assert_eq!(row.metrics.snr_db, None);
            assert_eq!(row.stddev_gain_bits, None);
        }
    }
}
