use adc_oversampling_enob::{
    adc::Adc,
    config::{ExperimentConfig, NoiseConfig, NoiseKind, SignalKind, SweepConfig},
    experiment, histogram, oversample, stats,
};

#[test]
fn quantizer_thresholds_and_resolution_bounds() {
    for bits in [2, 10, 12, 24] {
        let adc = Adc::new(bits, 1.0).unwrap();
        assert_eq!(adc.bits(), bits);
        assert_eq!(adc.v_ref(), 1.0);
        assert_eq!(adc.max_code(), (1u32 << bits) - 1);
        let half = 0.5 * adc.lsb();
        assert_eq!(adc.quantize(half * (1.0 - 1e-8)), 0);
        assert_eq!(adc.quantize(half * (1.0 + 1e-8)), 1);
        assert_eq!(adc.quantize(f64::NEG_INFINITY), 0);
        assert_eq!(adc.quantize(f64::INFINITY), adc.max_code());
        assert!(
            (adc.decode(adc.max_code() / 2) - (adc.max_code() / 2) as f64 * adc.lsb()).abs()
                < 1e-15
        );
    }
    assert!(Adc::new(12, 0.0).is_err());
    assert!(Adc::new(12, f64::INFINITY).is_err());
}

#[test]
fn one_sample_zero_error_and_clipping() {
    let mut c = ExperimentConfig {
        sample_count: 1,
        oversampling: vec![1],
        signal: SignalKind::Ramp {
            start: 0.0,
            end: 3.3,
        },
        noise: NoiseConfig {
            kind: NoiseKind::None,
            amplitude_lsb: 0.0,
        },
        ..Default::default()
    };
    let r = experiment::run(&c).unwrap();
    let m = &r.results[0];
    assert_eq!(m.metrics.rmse, 0.0);
    assert_eq!(m.rmse_gain_bits, None);
    assert_eq!(m.metrics.snr_db, None);
    c.sample_count = 256;
    c.oversampling = vec![1, 256];
    c.signal = SignalKind::Constant { voltage: -1.0 };
    let r = experiment::run(&c).unwrap();
    assert_eq!(r.raw_code_max, 0);
    assert_eq!(r.results[1].metrics.bias, 1.0);
    assert_eq!(r.results[1].metrics.rmse, 1.0);
}

#[test]
fn population_error_identity_with_bias() {
    let pairs = oversample::decimate(&[0.0, 1.0, 2.0], &[1.0, 3.0, 5.0], 1).unwrap();
    let m = stats::analyze(&pairs);
    assert_eq!(m.bias, 2.0);
    assert!((m.std_dev.powi(2) - 2.0 / 3.0).abs() < 1e-14);
    assert!((m.rmse.powi(2) - 14.0 / 3.0).abs() < 1e-14);
    assert!((m.rmse.powi(2) - m.bias.powi(2) - m.std_dev.powi(2)).abs() < 1e-14);
    assert!(stats::relative_gain(f64::INFINITY, 1.0).is_none());
    assert!(stats::relative_gain(-1.0, 1.0).is_none());
}

#[test]
fn invalid_configs_and_extreme_scales() {
    let mut c = ExperimentConfig {
        sample_count: 256,
        ..Default::default()
    };
    c.oversampling.clear();
    assert!(c.validated().is_err());
    c.oversampling = vec![512];
    assert!(c.validated().is_err());
    c.oversampling = vec![1];
    c.noise.amplitude_lsb = f64::INFINITY;
    assert!(c.validated().is_err());
    c.noise.amplitude_lsb = 0.0;
    c.signal = SignalKind::Constant { voltage: f64::NAN };
    assert!(c.validated().is_err());
    c.signal = SignalKind::Constant { voltage: 1.0 };
    c.v_ref = f64::MIN_POSITIVE / 4096.0;
    assert!(experiment::run(&c).is_err());
    let sweep = SweepConfig {
        amplitudes_lsb: vec![f64::NAN],
        ..Default::default()
    };
    assert!(experiment::run_sweep(&sweep).is_err());
    assert!(histogram::from_values(&[-f64::MAX, f64::MAX], 2).is_err());
}
