use crate::{experiment::ExperimentResult, histogram};
use std::fmt::Write;
pub type SvgError = String;
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
struct Canvas {
    body: String,
    min: f64,
    max: f64,
}
impl Canvas {
    fn new(title: &str, x_label: &str, y_label: &str, min: f64, max: f64) -> Self {
        let min = min.min(0.0);
        let max = if max <= min { min + 1.0 } else { max };
        let mut c = Self {
            body: format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1000\" height=\"600\" viewBox=\"0 0 1000 600\" role=\"img\">\n<title>{}</title>\n<rect width=\"1000\" height=\"600\" fill=\"#ffffff\"/>\n<g font-family=\"sans-serif\" font-size=\"14\" fill=\"#243447\">\n",
                escape(title)
            ),
            min,
            max,
        };
        c.text(500.0, 35.0, title, "middle");
        c.text(500.0, 580.0, x_label, "middle");
        writeln!(
            c.body,
            "<text transform=\"translate(22 290) rotate(-90)\" text-anchor=\"middle\">{}</text>",
            escape(y_label)
        )
        .unwrap();
        for i in 0..=5 {
            let value = min + (max - min) * i as f64 / 5.0;
            let y = c.y(value);
            writeln!(
                c.body,
                "<path d=\"M 110 {y:.3} H 950\" stroke=\"#e2e8f0\" fill=\"none\"/>"
            )
            .unwrap();
            c.text(98.0, y + 5.0, &format!("{value:.4}"), "end");
        }
        c.body
            .push_str("<path d=\"M 110 100 V 510 H 950\" stroke=\"#243447\" fill=\"none\"/>\n");
        c
    }
    fn y(&self, value: f64) -> f64 {
        510.0 - (value - self.min) / (self.max - self.min) * 410.0
    }
    fn text(&mut self, x: f64, y: f64, text: &str, anchor: &str) {
        writeln!(
            self.body,
            "<text x=\"{x:.3}\" y=\"{y:.3}\" text-anchor=\"{anchor}\">{}</text>",
            escape(text)
        )
        .unwrap();
    }
    fn line(&mut self, points: &[(f64, Option<f64>)], color: &str) {
        let mut path = String::new();
        let mut connected = false;
        for &(x, value) in points {
            if let Some(v) = value.filter(|v| v.is_finite()) {
                let y = self.y(v);
                write!(path, "{} {x:.3} {y:.3} ", if connected { "L" } else { "M" }).unwrap();
                connected = true;
                writeln!(
                    self.body,
                    "<circle cx=\"{x:.3}\" cy=\"{y:.3}\" r=\"4\" fill=\"{color}\"/>"
                )
                .unwrap();
            } else {
                connected = false;
            }
        }
        writeln!(
            self.body,
            "<path d=\"{path}\" stroke=\"{color}\" stroke-width=\"2.5\" fill=\"none\"/>"
        )
        .unwrap();
    }
    fn finish(mut self) -> String {
        self.body.push_str("</g>\n</svg>\n");
        self.body
    }
}
fn category(index: usize, count: usize) -> f64 {
    if count <= 1 {
        530.0
    } else {
        130.0 + 800.0 * index as f64 / (count - 1) as f64
    }
}
pub fn enob_vs_osr(result: &ExperimentResult) -> String {
    let values: Vec<_> = result
        .results
        .iter()
        .flat_map(|r| [r.stddev_gain_bits, Some(r.theoretical_gain_bits)])
        .flatten()
        .collect();
    let min = values.iter().copied().fold(0.0, f64::min);
    let max = values.iter().copied().fold(1.0, f64::max);
    let mut c = Canvas::new(
        "Effective-resolution gain vs oversampling",
        "Oversampling ratio (OSR; equally spaced categories)",
        "Gain relative to OSR 1 (bits)",
        min,
        max * 1.1,
    );
    c.text(130.0, 70.0, "Blue: observed StdDev gain", "start");
    c.text(520.0, 70.0, "Orange: ideal 0.5 log2(OSR)", "start");
    let mut observed = Vec::new();
    let mut ideal = Vec::new();
    for (i, r) in result.results.iter().enumerate() {
        let x = category(i, result.results.len());
        c.text(x, 540.0, &r.osr.to_string(), "middle");
        observed.push((x, r.stddev_gain_bits));
        ideal.push((x, Some(r.theoretical_gain_bits)));
    }
    c.line(&ideal, "#c65d12");
    c.line(&observed, "#1769aa");
    if observed.iter().all(|(_, v)| v.is_none()) {
        c.text(
            530.0,
            130.0,
            "Observed gain N/A: zero baseline or current spread",
            "middle",
        );
    }
    c.finish()
}
pub fn rmse_vs_osr(result: &ExperimentResult) -> String {
    let max = result
        .results
        .iter()
        .map(|r| r.metrics.rmse / result.adc_lsb_volts)
        .fold(0.0, f64::max);
    let mut c = Canvas::new(
        "RMS error vs oversampling",
        "Oversampling ratio (OSR; equally spaced categories)",
        "RMSE (original ADC LSB)",
        0.0,
        max * 1.1,
    );
    c.text(
        530.0,
        70.0,
        "RMSE includes random spread and systematic bias",
        "middle",
    );
    let points: Vec<_> = result
        .results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let x = category(i, result.results.len());
            c.text(x, 540.0, &r.osr.to_string(), "middle");
            (x, Some(r.metrics.rmse / result.adc_lsb_volts))
        })
        .collect();
    c.line(&points, "#1769aa");
    c.finish()
}
pub fn error_histogram(
    result: &ExperimentResult,
    osr: usize,
    bins: usize,
) -> Result<String, SvgError> {
    let r = result
        .results
        .iter()
        .find(|r| r.osr == osr)
        .ok_or_else(|| format!("histogram OSR {osr} is not present"))?;
    let h = histogram::from_values(&r.errors_lsb, bins)?;
    let peak = h.bins.iter().map(|b| b.count).max().unwrap_or(1) as f64;
    let mut c = Canvas::new(
        &format!("Error distribution at OSR {osr}"),
        "Error (original ADC LSB)",
        "Output sample count",
        0.0,
        peak * 1.1,
    );
    c.text(
        530.0,
        70.0,
        &format!("{} samples; {} fixed-width bins", r.output_samples, bins),
        "middle",
    );
    for (i, b) in h.bins.iter().enumerate() {
        let x = 110.0 + 840.0 * i as f64 / bins as f64;
        let width = 840.0 / bins as f64;
        let y = c.y(b.count as f64);
        let height = 510.0 - y;
        writeln!(c.body,"<rect x=\"{x:.3}\" y=\"{y:.3}\" width=\"{width:.3}\" height=\"{height:.3}\" fill=\"#1769aa\"><title>{:.6} to {:.6} LSB: {}</title></rect>",b.start,b.end,b.count).unwrap();
    }
    for i in 0..=4 {
        c.text(
            110.0 + 210.0 * i as f64,
            540.0,
            &format!("{:.4}", h.min + (h.max - h.min) * i as f64 / 4.0),
            "middle",
        );
    }
    Ok(c.finish())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escaped_and_finite() {
        assert_eq!(escape("<a&\"'>"), "&lt;a&amp;&quot;&apos;&gt;");
        let r = crate::experiment::run(&crate::config::ExperimentConfig {
            sample_count: 256,
            ..Default::default()
        })
        .unwrap();
        for svg in [
            enob_vs_osr(&r),
            rmse_vs_osr(&r),
            error_histogram(&r, 256, 80).unwrap(),
        ] {
            assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
            assert!(svg.ends_with("</svg>\n"));
            assert!(!svg.contains("NaN"));
            assert!(!svg.contains("Infinity"));
        }
        assert!(error_histogram(&r, 8, 80).is_err());
    }
}
