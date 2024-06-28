use crate::{raw_float::RawFloat, input::Input, lemire::BiasedFp, decimal::Decimal};


pub fn parse_long_mantissa<F: RawFloat, I: Input>(input: &I) -> BiasedFp {
    const MAX_SHIFT: usize = 60;
    const NUM_POWERS: usize = 19;
    const POWERS: [u8; 19] =
        [0, 3, 6, 9, 13, 16, 19, 23, 26, 29, 33, 36, 39, 43, 46, 49, 53, 56, 59];

    let get_shift = |n| {
        if n < NUM_POWERS { POWERS[n] as usize } else { MAX_SHIFT }
    };

    let mut d = Decimal::new(input);

    if d.num_digits == 0 || d.decimal_point < -324 {
        return BiasedFp::zero();
    } else if d.decimal_point >= 310 {
        return BiasedFp::inf::<F>();
    }

    let mut exp2 = 0_i32;
    // Shift right toward (1/2 ... 1].
    while d.decimal_point > 0 {
        let n = d.decimal_point as usize;
        let shift = get_shift(n);
        d.right_shift(shift);
        if d.decimal_point < -Decimal::DECIMAL_POINT_RANGE {
            return BiasedFp::zero();
        }
        exp2 += shift as i32;
    }
    // Shift left toward (1/2 ... 1].
    while d.decimal_point <= 0 {
        let shift = if d.decimal_point == 0 {
            match d.digits[0] {
                digit if digit >= 5 => break,
                0 | 1 => 2,
                _ => 1,
            }
        } else {
            get_shift((-d.decimal_point) as _)
        };
        d.left_shift(shift);
        if d.decimal_point > Decimal::DECIMAL_POINT_RANGE {
            return BiasedFp::inf::<F>();
        }
        exp2 -= shift as i32;
    }
    // We are now in the range [1/2 ... 1] but the binary format uses [1 ... 2].
    exp2 -= 1;
    while (F::MINIMUM_EXPONENT + 1) > exp2 {
        let mut n = ((F::MINIMUM_EXPONENT + 1) - exp2) as usize;
        if n > MAX_SHIFT {
            n = MAX_SHIFT;
        }
        d.right_shift(n);
        exp2 += n as i32;
    }
    if (exp2 - F::MINIMUM_EXPONENT) >= F::INFINITE_POWER {
        return BiasedFp::inf::<F>();
    }
    // Shift the decimal to the hidden bit, and then round the value
    // to get the high mantissa+1 bits.
    d.left_shift(F::MANTISSA_EXPLICIT_BITS + 1);
    let mut mantissa = d.round();
    if mantissa >= (1_u64 << (F::MANTISSA_EXPLICIT_BITS + 1)) {
        // Rounding up overflowed to the carry bit, need to
        // shift back to the hidden bit.
        d.right_shift(1);
        exp2 += 1;
        mantissa = d.round();
        if (exp2 - F::MINIMUM_EXPONENT) >= F::INFINITE_POWER {
            return BiasedFp::inf::<F>();
        }
    }
    let mut power2 = exp2 - F::MINIMUM_EXPONENT;
    if mantissa < (1_u64 << F::MANTISSA_EXPLICIT_BITS) {
        power2 -= 1;
    }
    // Zero out all the bits above the explicit mantissa bits.
    mantissa &= (1_u64 << F::MANTISSA_EXPLICIT_BITS) - 1;
    BiasedFp { f: mantissa, e: power2 }
}