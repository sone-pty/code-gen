use std::io;

use vnlex::ParseError;

#[derive(Debug)]
pub enum Error {
    IoErr(std::io::Error),
    FmtErr(std::fmt::Error),
    StringErr(StringErr),
    ParseIntErr(std::num::ParseIntError),
    ParseFloatErr(std::num::ParseFloatError),
    ParseLexErr(ParseError),
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::ParseLexErr(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IoErr(value)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Self::FmtErr(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::StringErr(StringErr(value))
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::StringErr(StringErr(value.into()))
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseIntErr(value)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::ParseFloatErr(value)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::IoErr(err) => err,
            Self::FmtErr(err) => err,
            Self::StringErr(err) => err,
            Self::ParseIntErr(err) => err,
            Self::ParseFloatErr(err) => err,
            Self::ParseLexErr(err) => err,
        })
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoErr(e) => write!(f, "{}", e),
            Self::FmtErr(e) => write!(f, "{}", e),
            Self::StringErr(e) => write!(f, "{}", e),
            Self::ParseIntErr(e) => write!(f, "{}", e),
            Self::ParseFloatErr(e) => write!(f, "{}", e),
            Self::ParseLexErr(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug)]
pub struct StringErr(String);

#[allow(dead_code)]
impl StringErr {
    pub fn new(content: &str) -> Self {
        Self { 0: content.into() }
    }
}

impl std::error::Error for StringErr {}

impl std::fmt::Display for StringErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
