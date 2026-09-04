use crate::{
    config::{NoiseKind, SignalKind},
    experiment::ExperimentResult,
};
use std::fmt::Write;
const HEADER: &str = "bits,vref,lsb_volts,signal,noise,noise_lsb,seed,input_samples,osr,output_samples,bias_lsb,stddev_lsb,rmse_lsb,snr_db,snr_enob,rmse_gain_bits,stddev_gain_bits,theoretical_gain_bits,raw_unique_codes\n";
pub fn noise_name(kind: NoiseKind) -> &'static str {
    match kind {
        NoiseKind::None => "none",
        NoiseKind::Uniform => "uniform",
        NoiseKind::Gaussian => "gaussian",
    }
}
fn optional(value: Option<f64>) -> String {
    value.map(|v| format!("{v:.12e}")).unwrap_or_default()
}
fn display(value: Option<f64>) -> String {
    value
        .map(|v| format!("{v:.6}"))
        .unwrap_or_else(|| "N/A".into())
}
pub fn console_text(result: &ExperimentResult) -> String {
    let c = &result.config;
    let lsb = result.adc_lsb_volts;
    let signal = match c.signal {
        SignalKind::Constant { voltage } => format!("constant {voltage:.9} V"),
        SignalKind::Ramp { start, end } => format!("ramp {start:.9} to {end:.9} V (inclusive)"),
    };
    let semantics = match c.noise.kind {
        NoiseKind::None => "disabled",
        NoiseKind::Uniform => "half-width",
        NoiseKind::Gaussian => "standard deviation",
    };
    let mut text = format!(
        "adc-oversampling-enob\nADC bits: {}\nVref: {:.9} V\nLSB: {:.12e} V ({:.9} mV)\nSignal: {signal}\nNoise: {} ({:.6} LSB {semantics})\nInput samples: {}\nSeed: {}\nRaw unique codes: {} (range {}..{})\n\n",
        c.bits,
        c.v_ref,
        lsb,
        lsb * 1000.0,
        noise_name(c.noise.kind),
        c.noise.amplitude_lsb,
        c.sample_count,
        c.seed,
        result.raw_unique_codes,
        result.raw_code_min,
        result.raw_code_max
    );
    text.push_str(" OSR | Output samples | Bias (LSB) | StdDev (LSB) | RMSE (LSB) | SNR (dB) | SNR ENOB | RMSE gain (bits) | StdDev gain (bits) | Ideal gain (bits)\n");
    for r in &result.results {
        let m = &r.metrics;
        writeln!(text,"{:4} | {:14} | {:10.6} | {:12.6} | {:10.6} | {:>8} | {:>8} | {:>16} | {:>18} | {:17.6}",r.osr,r.output_samples,m.bias/lsb,m.std_dev/lsb,m.rmse/lsb,display(m.snr_db),display(m.snr_enob),display(r.rmse_gain_bits),display(r.stddev_gain_bits),r.theoretical_gain_bits).unwrap();
    }
    text.push_str("\nIdeal independent-noise gain: 2x ~ +0.5 bit; 4x ~ +1 bit. Precision is not calibration or absolute accuracy.\n");
    if result.raw_unique_codes == 1 {
        text.push_str("One raw ADC code: repeated constant codes provide no threshold-crossing information.\n");
    }
    text
}
pub fn print_console(result: &ExperimentResult) {
    print!("{}", console_text(result));
}
fn rows(result: &ExperimentResult) -> String {
    let c = &result.config;
    let lsb = result.adc_lsb_volts;
    let signal = match c.signal {
        SignalKind::Constant { voltage } => format!("constant({voltage:.12e})"),
        SignalKind::Ramp { start, end } => format!("ramp({start:.12e};{end:.12e})"),
    };
    let mut text = String::new();
    for r in &result.results {
        let m = &r.metrics;
        writeln!(text,"{},{:.12e},{:.12e},{},{},{:.12e},{},{},{},{},{:.12e},{:.12e},{:.12e},{},{},{},{},{:.12e},{}",c.bits,c.v_ref,lsb,signal,noise_name(c.noise.kind),c.noise.amplitude_lsb,c.seed,c.sample_count,r.osr,r.output_samples,m.bias/lsb,m.std_dev/lsb,m.rmse/lsb,optional(m.snr_db),optional(m.snr_enob),optional(r.rmse_gain_bits),optional(r.stddev_gain_bits),r.theoretical_gain_bits,result.raw_unique_codes).unwrap();
    }
    text
}
pub fn summary_csv(result: &ExperimentResult) -> String {
    format!("{HEADER}{}", rows(result))
}
pub fn sweep_csv(results: &[ExperimentResult]) -> String {
    let mut text = HEADER.to_string();
    for r in results {
        text.push_str(&rows(r));
    }
    text
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_schema() {
        let r = crate::experiment::run(&crate::config::ExperimentConfig {
            sample_count: 256,
            ..Default::default()
        })
        .unwrap();
        let csv = summary_csv(&r);
        assert_eq!(csv.lines().count(), 6);
        for row in csv.lines() {
            assert_eq!(row.split(',').count(), 19);
        }
        assert!(console_text(&r).contains("N/A"));
        assert_eq!(summary_csv(&r), summary_csv(&r));
    }
}
