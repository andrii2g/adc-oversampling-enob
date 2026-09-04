mod common;
use common::{TempDir, run};
#[test]
fn help_and_version() {
    let dir = TempDir::new();
    for flag in ["--help", "--version"] {
        let out = run(&[flag], &dir);
        assert!(out.status.success());
        assert!(
            String::from_utf8(out.stdout)
                .unwrap()
                .contains("adc-oversampling-enob")
        );
    }
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 0);
}
#[test]
fn invalid_arguments() {
    for args in [
        vec!["--bits", "1"],
        vec!["--samples", "bad"],
        vec!["--samples", "0"],
        vec!["--bits"],
        vec!["--vref", "NaN"],
        vec!["--noise-lsb", "-1"],
        vec!["--osr", "0"],
        vec!["--osr", "3"],
        vec!["--samples", "257"],
        vec!["--histogram-bins", "1"],
        vec!["--wat"],
        vec!["run"],
        vec!["--noise", "pink"],
        vec!["--signal", "sine"],
        vec!["--ramp-end", "inf"],
    ] {
        let dir = TempDir::new();
        let out = run(&args, &dir);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(String::from_utf8(out.stderr).unwrap().contains("--help"));
        assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 0);
    }
}
#[test]
fn creates_artifacts_and_skips_absent_osrs() {
    let dir = TempDir::new();
    let out = run(&["--samples", "4096"], &dir);
    assert!(out.status.success(), "{:?}", out.stderr);
    for file in [
        "summary.csv",
        "enob-vs-osr.svg",
        "rmse-vs-osr.svg",
        "error-histogram-osr-1.svg",
        "error-histogram-osr-16.svg",
        "error-histogram-osr-256.svg",
    ] {
        assert!(dir.0.join(file).is_file(), "{file}");
    }
    let small = TempDir::new();
    assert!(
        run(&["--samples", "8", "--osr", "4"], &small)
            .status
            .success()
    );
    assert!(!small.0.join("error-histogram-osr-16.svg").exists());
    assert_eq!(
        std::fs::read_to_string(small.0.join("summary.csv"))
            .unwrap()
            .lines()
            .count(),
        3
    );
}
#[test]
fn sweep_rows_and_output_errors() {
    let dir = TempDir::new();
    let out = run(&["sweep", "--samples", "1024"], &dir);
    assert!(out.status.success());
    let csv = std::fs::read_to_string(dir.0.join("sweep.csv")).unwrap();
    assert_eq!(csv.lines().count(), 41);
    let file = dir.0.join("not-a-directory");
    std::fs::write(&file, "keep").unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_adc-oversampling-enob"))
        .args(["--samples", "256", "--out"])
        .arg(&file)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().starts_with("error:"));
    assert_eq!(std::fs::read_to_string(file).unwrap(), "keep");
}
