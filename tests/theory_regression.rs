use adc_oversampling_enob::{
    adc::Adc,
    config::{ExperimentConfig, NoiseConfig, NoiseKind, SignalKind, SweepConfig},
    experiment, oversample, signal, stats,
};
#[test]
fn ideal_gain() {
    for (osr, gain) in [
        (1, 0.0),
        (2, 0.5),
        (4, 1.0),
        (16, 2.0),
        (64, 3.0),
        (256, 4.0),
    ] {
        assert_eq!(stats::theoretical_enob_gain(osr), gain);
    }
}
#[test]
fn no_dither_preserves_offset() {
    let c = ExperimentConfig {
        sample_count: 16384,
        noise: NoiseConfig {
            kind: NoiseKind::None,
            amplitude_lsb: 0.0,
        },
        ..Default::default()
    };
    let r = experiment::run(&c).unwrap();
    assert_eq!(r.raw_unique_codes, 1);
    assert!(r.results[0].metrics.rmse > 0.0);
    for row in &r.results {
        assert_eq!(row.metrics.rmse, r.results[0].metrics.rmse);
        assert_eq!(row.metrics.bias, r.results[0].metrics.bias);
        assert_eq!(row.metrics.std_dev, 0.0);
        assert_eq!(row.rmse_gain_bits, Some(0.0));
        assert_eq!(row.stddev_gain_bits, None);
        assert_eq!(row.metrics.snr_db, None);
    }
}
#[test]
fn gaussian_reduction_tracks_theory() {
    let r = experiment::run(&ExperimentConfig {
        sample_count: 262144,
        ..Default::default()
    })
    .unwrap();
    assert!(r.raw_unique_codes > 2);
    for row in &r.results {
        assert!((row.stddev_gain_bits.unwrap() - row.theoretical_gain_bits).abs() < 0.15);
        assert!(
            (row.metrics.rmse.powi(2) - row.metrics.std_dev.powi(2) - row.metrics.bias.powi(2))
                .abs()
                < 1e-18
        );
    }
    assert!(r.results.last().unwrap().metrics.std_dev < r.results[0].metrics.std_dev / 12.0);
}
#[test]
fn uniform_threshold_crossing() {
    let c = ExperimentConfig {
        sample_count: 131072,
        noise: NoiseConfig {
            kind: NoiseKind::Uniform,
            amplitude_lsb: 0.5,
        },
        ..Default::default()
    };
    let r = experiment::run(&c).unwrap();
    assert_eq!(r.raw_unique_codes, 2);
    assert!(r.results.last().unwrap().rmse_gain_bits.unwrap() > 3.5);
}
#[test]
fn ramp_group_mean_and_snr() {
    let c = ExperimentConfig {
        sample_count: 1024,
        signal: SignalKind::Ramp {
            start: 0.1,
            end: 3.2,
        },
        ..Default::default()
    };
    let r = experiment::run(&c).unwrap();
    assert!(r.results.iter().all(|r| r.metrics.snr_db.is_some()));
    let reference = signal::generate(c.signal, c.sample_count);
    let adc = Adc::new(c.bits, c.v_ref).unwrap();
    let measured: Vec<_> = reference
        .iter()
        .map(|&v| adc.quantize_and_decode(v).1)
        .collect();
    let d = oversample::decimate(&reference, &measured, 256).unwrap();
    assert!((d[0].reference - (reference[0] + reference[255]) / 2.0).abs() < 1e-14);
    assert!((d[0].measured - d[0].reference).abs() < adc.lsb());
    let flat = experiment::run(&ExperimentConfig {
        signal: SignalKind::Ramp {
            start: 1.0,
            end: 1.0,
        },
        ..c
    })
    .unwrap();
    assert!(flat.results.iter().all(|r| r.metrics.snr_db.is_none()));
}
#[test]
fn sweep_restarts_seed() {
    let c = SweepConfig {
        base: ExperimentConfig {
            sample_count: 1024,
            ..Default::default()
        },
        ..Default::default()
    };
    let sweep = experiment::run_sweep(&c).unwrap();
    assert_eq!(sweep.len(), 8);
    for row in sweep {
        let mut single = c.base.clone();
        single.noise.amplitude_lsb = row.config.noise.amplitude_lsb;
        let expected = experiment::run(&single).unwrap();
        assert_eq!(row.results[0].errors_lsb, expected.results[0].errors_lsb);
    }
}
