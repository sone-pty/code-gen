use std::iter::FusedIterator;

use crate::digit::Digit;


pub trait Input {
    type IntegerDigits<'a>: DoubleEndedIterator<Item = Digit> where Self: 'a;
    type FractionDigits<'a>: DoubleEndedIterator<Item = Digit> where Self: 'a;
    
    fn exponent(&self) -> i16;
    fn integer_digits(&self) -> Self::IntegerDigits<'_>;
    fn fraction_digits(&self) -> Self::FractionDigits<'_>;
}

pub struct Literal<'a> {
    pub integer: &'a str,
    pub fraction: &'a str,
    pub exponent: i16,
}

impl<'a> Input for Literal<'a> {
    type IntegerDigits<'r> = LiteralDights<'r> where Self: 'r;
    type FractionDigits<'r> = LiteralDights<'r> where Self: 'r;
    fn exponent(&self) -> i16 {
        self.exponent
    }
    fn integer_digits(&self) -> Self::IntegerDigits<'_> {
        LiteralDights (self.integer.as_bytes().iter())
    }
    fn fraction_digits(&self) -> Self::FractionDigits<'_> {
        LiteralDights (self.fraction.as_bytes().iter())
    }
}

impl<'a> Literal<'a> {
    pub fn new(integer: &'a str, fraction: &'a str, exponent: i16) -> Self {
        Literal { integer, fraction, exponent }
    }
}

pub struct LiteralDights<'a> (std::slice::Iter<'a, u8>);

impl<'a> Iterator for LiteralDights<'a> {
    type Item = Digit;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ch) = self.0.next() {
            if let Some(digit) = Digit::from_ascii(*ch) {
                return Some(digit);
            }
        }
        None
    }
}
impl<'a> DoubleEndedIterator for LiteralDights<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(ch) = self.0.next_back() {
            if let Some(digit) = Digit::from_ascii(*ch) {
                return Some(digit);
            }
        }
        None
    }
}

impl FusedIterator for LiteralDights<'_> {}