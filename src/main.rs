use adc_oversampling_enob::{
    cli::{self, Command},
    experiment, report, svg,
};
fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    match cli::parse(std::env::args().skip(1))? {
        Command::Help => print!("{}", cli::help_text()),
        Command::Version => println!("adc-oversampling-enob {}", env!("CARGO_PKG_VERSION")),
        Command::Run(c) => {
            let result = experiment::run(&c)?;
            let mut artifacts = vec![
                ("enob-vs-osr.svg".to_string(), svg::enob_vs_osr(&result)),
                ("rmse-vs-osr.svg".to_string(), svg::rmse_vs_osr(&result)),
            ];
            for osr in [1, 16, 256] {
                if result.results.iter().any(|r| r.osr == osr) {
                    artifacts.push((
                        format!("error-histogram-osr-{osr}.svg"),
                        svg::error_histogram(&result, osr, c.histogram_bins)?,
                    ));
                }
            }
            std::fs::create_dir_all(&c.output_dir)?;
            for (name, contents) in artifacts {
                std::fs::write(c.output_dir.join(name), contents)?;
            }
            std::fs::write(
                c.output_dir.join("summary.csv"),
                report::summary_csv(&result),
            )?;
            report::print_console(&result);
        }
        Command::Sweep(c) => {
            let results = experiment::run_sweep(&c)?;
            std::fs::create_dir_all(&c.base.output_dir)?;
            std::fs::write(
                c.base.output_dir.join("sweep.csv"),
                report::sweep_csv(&results),
            )?;
            for result in &results {
                report::print_console(result);
            }
        }
    }
    Ok(())
}
