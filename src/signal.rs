use crate::config::SignalKind;
pub fn sample(kind: SignalKind, index: usize, count: usize) -> f64 {
    match kind {
        SignalKind::Constant { voltage } => voltage,
        SignalKind::Ramp { start, end } => {
            if count <= 1 || index == 0 {
                start
            } else if index == count - 1 {
                end
            } else {
                start + index as f64 / (count - 1) as f64 * (end - start)
            }
        }
    }
}
pub fn generate(kind: SignalKind, count: usize) -> Vec<f64> {
    (0..count).map(|i| sample(kind, i, count)).collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoints() {
        assert_eq!(
            generate(SignalKind::Constant { voltage: 1.2 }, 3),
            vec![1.2; 3]
        );
        let r = SignalKind::Ramp {
            start: 0.1,
            end: 3.2,
        };
        let v = generate(r, 3);
        assert_eq!(v[0], 0.1);
        assert_eq!(v[2], 3.2);
        assert!((v[1] - 1.65).abs() < 1e-14);
        assert_eq!(generate(r, 1), [0.1]);
    }
}
