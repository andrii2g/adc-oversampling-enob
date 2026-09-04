#[derive(Debug, Clone, Copy)]
pub struct Adc {
    bits: u8,
    v_ref: f64,
    max_code: u32,
}
pub type AdcError = String;
impl Adc {
    pub fn new(bits: u8, v_ref: f64) -> Result<Self, AdcError> {
        if !(2..=24).contains(&bits) || !v_ref.is_finite() || v_ref <= 0.0 {
            return Err("invalid ADC bits or Vref".into());
        }
        Ok(Self {
            bits,
            v_ref,
            max_code: (1u32 << bits) - 1,
        })
    }
    pub fn bits(&self) -> u8 {
        self.bits
    }
    pub fn v_ref(&self) -> f64 {
        self.v_ref
    }
    pub fn max_code(&self) -> u32 {
        self.max_code
    }
    pub fn lsb(&self) -> f64 {
        self.v_ref / self.max_code as f64
    }
    pub fn quantize(&self, volts: f64) -> u32 {
        (volts.clamp(0.0, self.v_ref) / self.v_ref * self.max_code as f64).round() as u32
    }
    pub fn decode(&self, code: u32) -> f64 {
        code as f64 / self.max_code as f64 * self.v_ref
    }
    pub fn quantize_and_decode(&self, volts: f64) -> (u32, f64) {
        let code = self.quantize(volts);
        (code, self.decode(code))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries_and_monotonicity() {
        let a = Adc::new(12, 3.3).unwrap();
        assert_eq!(a.quantize(-1.0), 0);
        assert_eq!(a.quantize(0.0), 0);
        assert_eq!(a.quantize(3.3), 4095);
        assert_eq!(a.quantize(4.0), 4095);
        assert_eq!(a.decode(4095), 3.3);
        assert_eq!(a.lsb(), 3.3 / 4095.0);
        for i in 0..10000 {
            assert!(a.quantize(i as f64 / 3000.0) <= a.quantize((i + 1) as f64 / 3000.0));
        }
        assert!(Adc::new(25, 3.3).is_err());
    }
}
