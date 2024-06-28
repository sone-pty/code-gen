
use input::Input;
use lemire::BiasedFp;
use number::Number;
use raw_float::RawFloat;
use slow::parse_long_mantissa;


pub mod digit;
pub mod input;
pub mod raw_float;

mod number;
mod fpu;
mod lemire;
mod table;
mod slow;
mod decimal;


pub fn dec2flt<F: RawFloat, I: Input>(input: &I) -> F {
    let num = Number::new(input);
    let fp = 
        if !num.many_digits {
            if let Some(value) = num.try_fast_path() {
                return value;
            }
            BiasedFp::compute::<F>(num.exponent, num.mantissa)
                .unwrap_or_else(|| parse_long_mantissa::<F, _>(input))
        } else if let Some(t) = BiasedFp::compute::<F>(num.exponent, num.mantissa) {
            if BiasedFp::compute::<F>(num.exponent, num.mantissa + 1) != Some(t) {
                parse_long_mantissa::<F, _>(input)
            } else {
                t
            }
        } else {
            parse_long_mantissa::<F, _>(input)
        };

    let mut word = fp.f;
    word |= (fp.e as u64) << F::MANTISSA_EXPLICIT_BITS;
    F::from_u64_bits(word)
}

pub fn dec2flt_slow<F: RawFloat, I: Input>(input: &I) -> F {
    let fp = parse_long_mantissa::<F, _>(input);
    let mut word = fp.f;
    word |= (fp.e as u64) << F::MANTISSA_EXPLICIT_BITS;
    F::from_u64_bits(word)
}



#[cfg(test)]
mod tests {
    use crate::{input::Literal, dec2flt, dec2flt_slow};

    #[test]
    fn short_float() {
        let short_float = Literal::new("167772170", "", -1);
        assert_eq!(167772170e-1_f32, dec2flt(&short_float));
        assert_eq!(167772170e-1_f64, dec2flt(&short_float));
    }

    #[test]
    fn long_float() {

        let long_float = Literal::new("1234567890", "12345678900", -3);

        assert_eq!(1234567890.12345678900e-3_f32, dec2flt(&long_float));
        assert_eq!(1234567890.12345678900e-3, dec2flt(&long_float));

    }

    #[test]
    fn slow() {
        let long_float = Literal::new("1234567890", "12345678900", -3);
        assert_eq!(1234567890.12345678900e-3_f32, dec2flt_slow(&long_float));
        assert_eq!(1234567890.12345678900e-3_f64, dec2flt_slow(&long_float));

        let short_float = Literal::new("167772170", "", 1);
        assert_eq!(167772170e+1_f32, dec2flt_slow(&short_float));
        assert_eq!(167772170e+1_f64, dec2flt_slow(&short_float));
    }
}
