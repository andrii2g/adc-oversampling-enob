#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBin {
    pub start: f64,
    pub end: f64,
    pub count: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Histogram {
    pub min: f64,
    pub max: f64,
    pub bins: Vec<HistogramBin>,
}
pub type HistogramError = String;
pub fn from_values(values: &[f64], bin_count: usize) -> Result<Histogram, HistogramError> {
    if values.is_empty() || bin_count < 2 || values.iter().any(|v| !v.is_finite()) {
        return Err("histogram requires finite nonempty values and at least two bins".into());
    }
    let mut min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let mut max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if min == max {
        let padding = (min.abs() * 1e-6).max(0.5);
        min -= padding;
        max += padding;
    }
    let width = (max - min) / bin_count as f64;
    if !min.is_finite() || !max.is_finite() || !width.is_finite() || width <= 0.0 {
        return Err("histogram range exceeds finite numeric precision".into());
    }
    let mut bins: Vec<_> = (0..bin_count)
        .map(|i| HistogramBin {
            start: min + i as f64 * width,
            end: if i + 1 == bin_count {
                max
            } else {
                min + (i + 1) as f64 * width
            },
            count: 0,
        })
        .collect();
    for &value in values {
        let index = (((value - min) / width) as usize).min(bin_count - 1);
        bins[index].count += 1;
    }
    Ok(Histogram { min, max, bins })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_and_boundaries() {
        let h = from_values(&[0.0, 1.0, 2.0, 3.0, 4.0], 4).unwrap();
        assert_eq!(
            h.bins.iter().map(|b| b.count).collect::<Vec<_>>(),
            [1, 1, 1, 2]
        );
        let h = from_values(&[0.2; 10], 80).unwrap();
        assert!(h.min < 0.2 && h.max > 0.2);
        assert_eq!(h.bins.iter().map(|b| b.count).sum::<usize>(), 10);
        assert!(from_values(&[], 4).is_err());
        assert!(from_values(&[f64::NAN], 4).is_err());
        assert!(from_values(&[1.0], 1).is_err());
    }
}
