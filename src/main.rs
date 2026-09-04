use adc_oversampling_enob::{
    cli::{self, Command},
    experiment, report,
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
            std::fs::create_dir_all(&c.output_dir)?;
            std::fs::write(
                c.output_dir.join("summary.csv"),
                report::summary_csv(&result),
            )?;
            report::print_console(&result);
        }
        Command::Sweep(_) => return Err("sweep output is not implemented yet".into()),
    }
    Ok(())
}
