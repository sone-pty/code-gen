#[allow(dead_code)]
#[derive(Debug)]
pub enum CData<'a> {
    Digits(&'a str),
    DecimalDigitsWithExponent(&'a str, bool, &'a str),
    Char(char),
}

#[allow(dead_code)]
impl<'a> CData<'a> {
    pub fn into_char(self) -> Option<char> {
        match self {
            CData::Char(t) => Some(t),
            _ => None,
        }
    }

    pub fn into_digits(self) -> Option<&'a str> {
        match self {
            CData::Digits(t) => Some(t),
            _ => None,
        }
    }

    pub fn into_int_exp(self) -> Option<(&'a str, bool, &'a str)> {
        match self {
            CData::DecimalDigitsWithExponent(t1, t2, t3) => Some((t1, t2, t3)),
            _ => None,
        }
    }
}
