#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecimatedSample {
    pub reference: f64,
    pub measured: f64,
}
pub type OversampleError = String;
/// Sum deviations from the first value: constant blocks remain exactly constant.
fn mean(values: &[f64]) -> f64 {
    let origin = values[0];
    origin
        + values
            .iter()
            .map(|x| (x - origin) / values.len() as f64)
            .sum::<f64>()
}
pub fn decimate(
    reference: &[f64],
    measured: &[f64],
    factor: usize,
) -> Result<Vec<DecimatedSample>, OversampleError> {
    if reference.len() != measured.len() || factor == 0 || !reference.len().is_multiple_of(factor) {
        return Err(
            "averaging requires equal lengths and a positive factor dividing the length".into(),
        );
    }
    Ok(reference
        .chunks_exact(factor)
        .zip(measured.chunks_exact(factor))
        .map(|(r, m)| DecimatedSample {
            reference: mean(r),
            measured: mean(m),
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matched_windows() {
        let r = [1.0, 2.0, 3.0, 4.0];
        let m = [2.0, 4.0, 6.0, 8.0];
        assert_eq!(
            decimate(&r, &m, 4).unwrap(),
            [DecimatedSample {
                reference: 2.5,
                measured: 5.0
            }]
        );
        assert_eq!(
            decimate(&r, &m, 1).unwrap()[2],
            DecimatedSample {
                reference: 3.0,
                measured: 6.0
            }
        );
        assert!(decimate(&r, &m, 0).is_err());
        assert!(decimate(&r, &m, 3).is_err());
        assert!(decimate(&r, &m[..2], 2).is_err());
    }
    #[test]
    fn constant_exact() {
        assert_eq!(
            decimate(&[1.234567; 256], &[1.235; 256], 256).unwrap()[0],
            DecimatedSample {
                reference: 1.234567,
                measured: 1.235
            }
        );
    }
}
