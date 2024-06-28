use std::fmt;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Digit(u8);

impl Digit {
    pub fn from_u8(val: u8) -> Option<Digit> {
        if val < 10 {
            Some(Digit(val))
        } else {
            None
        }
    }

    pub fn from_ascii(ch: u8) -> Option<Digit> {
        Self::from_u8(ch.wrapping_sub(b'0'))
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}