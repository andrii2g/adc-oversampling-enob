/// SplitMix64 with wrapping arithmetic and a 53-bit uniform conversion.
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}
impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_sequence() {
        let mut r = SplitMix64::new(0);
        assert_eq!(r.next_u64(), 0xe220a8397b1dcdaf);
        assert_eq!(r.next_u64(), 0x6e789e6aa1b965f4);
        assert_eq!(r.next_u64(), 0x06c45d188009454f);
    }
    #[test]
    fn reproducible_uniform() {
        let mut a = SplitMix64::new(42);
        let mut b = a.clone();
        for _ in 0..10000 {
            let x = a.next_f64();
            assert_eq!(x, b.next_f64());
            assert!((0.0..1.0).contains(&x));
        }
        assert_ne!(a.next_u64(), SplitMix64::new(43).next_u64());
    }
}
