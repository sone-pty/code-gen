use crate::{raw_float::RawFloat, fpu::set_precision, input::Input};


const INT_POW10: [u64; 16] = [
    1,
    10,
    100,
    1000,
    10000,
    100000,
    1000000,
    10000000,
    100000000,
    1000000000,
    10000000000,
    100000000000,
    1000000000000,
    10000000000000,
    100000000000000,
    1000000000000000,
];

const MIN_19DIGIT_INT: u64 = 100_0000_0000_0000_0000;


#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Number {
    pub exponent: i64,
    pub mantissa: u64,
    pub many_digits: bool,
}

impl Number {
    fn is_fast_path<F: RawFloat>(&self) -> bool {
        F::MIN_EXPONENT_FAST_PATH <= self.exponent
        && self.exponent <= F::MAX_EXPONENT_DISGUISED_FAST_PATH
        && self.mantissa <= F::MAX_MANTISSA_FAST_PATH
    }

    pub fn try_fast_path<F: RawFloat>(&self) -> Option<F> {
        if self.is_fast_path::<F>() {
            let _cw = set_precision::<F>();
            Some(if self.exponent <= F::MAX_EXPONENT_FAST_PATH {
                let value = F::from_u64(self.mantissa);
                if self.exponent < 0 {
                    value / F::pow10_fast_path((-self.exponent) as _)
                } else {
                    value * F::pow10_fast_path(self.exponent as _)
                }
            } else {
                let shift = self.exponent - F::MAX_EXPONENT_FAST_PATH;
                let mantissa = self.mantissa.checked_mul(INT_POW10[shift as usize])?;
                if mantissa > F::MAX_MANTISSA_FAST_PATH {
                    return None;
                }
                F::from_u64(mantissa) * F::pow10_fast_path(F::MAX_EXPONENT_FAST_PATH as _)
            })
        } else {
            None
        }
    }

    pub fn new<I: Input>(input: &I) -> Number {
        let mut num = Self::default();
        let mut digits = input.integer_digits();
        while let Some(digit) = digits.next() {
            num.mantissa = num.mantissa * 10 + digit.value() as u64;
            if num.mantissa >= MIN_19DIGIT_INT {
                num.exponent = digits.count() as i64;
                num.many_digits = if num.exponent == 0 {
                    input.fraction_digits().next().is_some()
                } else {
                    true
                };
                num.exponent += input.exponent() as i64;
                return num;
            }
        }

        let mut digits = input.fraction_digits();
        while let Some(digit) = digits.next() {
            num.mantissa = num.mantissa * 10 + digit.value() as u64;
            num.exponent -= 1;
            if num.mantissa >= MIN_19DIGIT_INT {
                num.many_digits = digits.next().is_some();
                break;
            }
        }
        num.exponent += input.exponent() as i64;
        num
    }

}


