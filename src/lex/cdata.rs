#[allow(dead_code)]
#[derive(Debug)]
pub enum CData<'a> {
    Digits(&'a str, bool),
    DecimalDigitsWithExponent(&'a str, bool, &'a str, bool),
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

    pub fn into_digits(&self) -> Option<(&'a str, bool)> {
        match self {
            CData::Digits(t, minus) => Some((t, *minus)),
            _ => None,
        }
    }

    pub fn into_int_exp(&self) -> Option<(&'a str, bool, &'a str, bool)> {
        match self {
            CData::DecimalDigitsWithExponent(t1, t2, t3, minus) => Some((t1, *t2, t3, *minus)),
            _ => None,
        }
    }
}
