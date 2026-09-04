mod common;
use adc_oversampling_enob::{config::ExperimentConfig, experiment, report, svg};
use common::{TempDir, run};
#[test]
fn identical_data_and_strings() {
    let mut c = ExperimentConfig {
        sample_count: 4096,
        ..Default::default()
    };
    let a = experiment::run(&c).unwrap();
    let b = experiment::run(&c).unwrap();
    assert_eq!(
        (a.raw_code_min, a.raw_code_max, a.raw_unique_codes),
        (b.raw_code_min, b.raw_code_max, b.raw_unique_codes)
    );
    for (x, y) in a.results.iter().zip(&b.results) {
        assert_eq!(x.metrics, y.metrics);
        assert_eq!(x.errors_lsb, y.errors_lsb);
    }
    assert_eq!(report::summary_csv(&a), report::summary_csv(&b));
    assert_eq!(report::console_text(&a), report::console_text(&b));
    assert_eq!(svg::enob_vs_osr(&a), svg::enob_vs_osr(&b));
    assert_eq!(svg::rmse_vs_osr(&a), svg::rmse_vs_osr(&b));
    for osr in [1, 16, 256] {
        assert_eq!(
            svg::error_histogram(&a, osr, 80).unwrap(),
            svg::error_histogram(&b, osr, 80).unwrap()
        );
    }
    c.seed += 1;
    assert_ne!(
        report::summary_csv(&a),
        report::summary_csv(&experiment::run(&c).unwrap())
    );
    assert_ne!(
        a.results[0].metrics,
        experiment::run(&c).unwrap().results[0].metrics
    );
}
#[test]
fn subprocess_bytes_and_overwrite() {
    let a = TempDir::new();
    let b = TempDir::new();
    let first = run(&["--samples", "4096"], &a);
    let second = run(&["--samples", "4096"], &b);
    assert!(first.status.success() && second.status.success());
    assert_eq!(first.stdout, second.stdout);
    for entry in std::fs::read_dir(&a.0).unwrap() {
        let entry = entry.unwrap();
        assert_eq!(
            std::fs::read(entry.path()).unwrap(),
            std::fs::read(b.0.join(entry.file_name())).unwrap()
        );
    }
    assert_eq!(first.stdout, run(&["--samples", "4096"], &a).stdout);
    let first = run(&["sweep", "--samples", "1024"], &a);
    let second = run(&["sweep", "--samples", "1024"], &b);
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(
        std::fs::read(a.0.join("sweep.csv")).unwrap(),
        std::fs::read(b.0.join("sweep.csv")).unwrap()
    );
}
