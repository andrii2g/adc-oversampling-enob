use crate::config::{ExperimentConfig, NoiseKind, SignalKind, SweepConfig};
use std::{fmt, str::FromStr};
#[derive(Debug)]
pub enum Command {
    Run(ExperimentConfig),
    Sweep(SweepConfig),
    Help,
    Version,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub message: String,
}
impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}; use --help for usage", self.message)
    }
}
impl std::error::Error for CliError {}
fn error(message: impl Into<String>) -> CliError {
    CliError {
        message: message.into(),
    }
}
fn number<T: FromStr>(option: &str, value: &str) -> Result<T, CliError> {
    value
        .parse()
        .map_err(|_| error(format!("invalid value '{value}' for {option}")))
}
pub fn parse<I, S>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).peekable();
    let sweep = args.peek().is_some_and(|s| s == "sweep");
    if sweep {
        args.next();
    }
    let mut c = ExperimentConfig::default();
    let mut signal = "constant".to_string();
    let mut voltage: f64 = 1.234567;
    let mut start: f64 = 0.1;
    let mut end: f64 = 3.2;
    while let Some(option) = args.next() {
        if option == "--help" {
            return Ok(Command::Help);
        }
        if option == "--version" {
            return Ok(Command::Version);
        }
        if !matches!(
            option.as_str(),
            "--bits"
                | "--vref"
                | "--samples"
                | "--seed"
                | "--signal"
                | "--voltage"
                | "--ramp-start"
                | "--ramp-end"
                | "--noise"
                | "--noise-lsb"
                | "--osr"
                | "--out"
                | "--histogram-bins"
        ) {
            return Err(error(format!("unknown option or subcommand '{option}'")));
        }
        let value = args
            .next()
            .filter(|v| !v.starts_with("--"))
            .ok_or_else(|| error(format!("missing value for {option}")))?;
        match option.as_str() {
            "--bits" => c.bits = number(&option, &value)?,
            "--vref" => c.v_ref = number(&option, &value)?,
            "--samples" => c.sample_count = number(&option, &value)?,
            "--seed" => c.seed = number(&option, &value)?,
            "--voltage" => voltage = number(&option, &value)?,
            "--ramp-start" => start = number(&option, &value)?,
            "--ramp-end" => end = number(&option, &value)?,
            "--signal" => {
                if !matches!(value.as_str(), "constant" | "ramp") {
                    return Err(error(format!("invalid --signal '{value}'")));
                }
                signal = value;
            }
            "--noise" => {
                c.noise.kind = match value.as_str() {
                    "none" => NoiseKind::None,
                    "uniform" => NoiseKind::Uniform,
                    "gaussian" => NoiseKind::Gaussian,
                    _ => return Err(error(format!("invalid --noise '{value}'"))),
                }
            }
            "--noise-lsb" => c.noise.amplitude_lsb = number(&option, &value)?,
            "--osr" => {
                c.oversampling = value
                    .split(',')
                    .map(|v| number(&option, v))
                    .collect::<Result<_, _>>()?
            }
            "--out" => {
                if value.is_empty() {
                    return Err(error("--out must not be empty"));
                }
                c.output_dir = value.into();
            }
            "--histogram-bins" => c.histogram_bins = number(&option, &value)?,
            _ => unreachable!(),
        }
    }
    for (option, value) in [
        ("--voltage", voltage),
        ("--ramp-start", start),
        ("--ramp-end", end),
    ] {
        if !value.is_finite() {
            return Err(error(format!("{option} must be finite, got {value}")));
        }
    }
    c.signal = if signal == "constant" {
        SignalKind::Constant { voltage }
    } else {
        SignalKind::Ramp { start, end }
    };
    let c = c.validated().map_err(error)?;
    Ok(if sweep {
        Command::Sweep(SweepConfig {
            base: c,
            ..Default::default()
        })
    } else {
        Command::Run(c)
    })
}
pub fn help_text() -> &'static str {
    "adc-oversampling-enob [sweep] [OPTIONS]\n\nSimulate ideal ADC quantization, analog dither, and block averaging.\n\nOptions (defaults in parentheses):\n  --bits <2..=24>                 ADC resolution (12)\n  --vref <volts>                  Positive reference voltage (3.3)\n  --samples <count>               Raw samples (1048576)\n  --seed <u64>                    SplitMix64 seed (42)\n  --signal <constant|ramp>        Signal (constant)\n  --voltage <volts>               Constant voltage (1.234567)\n  --ramp-start <volts>            Inclusive ramp start (0.1)\n  --ramp-end <volts>              Inclusive ramp end (3.2)\n  --noise <none|uniform|gaussian> Noise (gaussian)\n  --noise-lsb <amplitude>         Gaussian sigma / uniform half-width (0.75)\n  --osr <powers-of-two>           Comma-separated factors (1,4,16,64,256)\n  --out <directory>              Output directory (results)\n  --histogram-bins <count>        Histogram bins, at least 2 (80)\n  --help                         Print help\n  --version                      Print version\n\nOSRs are sorted/deduplicated; 1 is inserted as baseline. Every factor must\ndivide the sample count. Sweep overrides --noise-lsb with amplitudes\n0,0.05,0.10,0.25,0.50,0.75,1.00,2.00 and reuses the seed per run.\n"
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parsing() {
        assert!(matches!(parse(["--help"]), Ok(Command::Help)));
        assert!(matches!(parse(["sweep"]), Ok(Command::Sweep(_))));
        for args in [
            vec!["--bits", "x"],
            vec!["--bits", "1"],
            vec!["--bits"],
            vec!["--wat"],
            vec!["run"],
            vec!["--osr", "3"],
            vec!["--voltage", "NaN"],
        ] {
            assert!(parse(args).unwrap_err().to_string().contains("--help"));
        }
        let Command::Run(c) =
            parse(["--signal", "ramp", "--ramp-end", "2", "--osr", "16,4,16"]).unwrap()
        else {
            panic!()
        };
        assert_eq!(c.oversampling, [1, 4, 16]);
        assert_eq!(
            c.signal,
            SignalKind::Ramp {
                start: 0.1,
                end: 2.0
            }
        );
    }
}
