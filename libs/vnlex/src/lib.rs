//#![feature(str_internals)]
#![feature(str_from_raw_parts)]
#![feature(ptr_sub_ptr)]

#![feature(unsize)]
#![feature(hash_set_entry)]

use std::{path::PathBuf, fmt, error::Error, borrow::Cow, sync::Arc, ops::Range};

use cursor::Cursor;

pub mod cursor;
pub mod token;
pub mod lexer;
pub mod syntax_builder;
pub mod syntaxer;
pub use dec2flt;


#[derive(Debug, Clone)]
pub struct Location {
    pub file: Option<Arc<PathBuf>>,
    pub row: usize,
    pub col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl Location {
    pub const DEFAULT: Self = Self { file: None, row: 0, col: 0, end_row: 0, end_col: 0 };
    pub fn new(file: Option<Arc<PathBuf>>, range: Range<(usize, usize)>) -> Self {
        let Range { start: (row, col), end: (end_row, end_col) } = range;
        Self {
            file,
            row,
            col,
            end_row,
            end_col,
        }
    }

    pub fn start(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    pub fn end(&self) -> (usize, usize) {
        (self.end_row, self.end_col)
    }

    pub fn contains(&self, row: usize, col: usize) -> bool {
        if self.row > row || self.row == row && self.col > col {
            false
        } else if self.end_row <= row || self.end_row == row && self.end_col <= col {
            false
        } else {
            true
        }
    }

    pub fn display_end(&self) -> LocationDisplayEnd {
        LocationDisplayEnd(self)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref file) = self.file {
            write!(f, "{}", file.display())?;
        }
        write!(f, ":{}:{}", self.row + 1, self.col + 1)
    }
}

pub struct LocationDisplayEnd<'r> (&'r Location);

impl fmt::Display for LocationDisplayEnd<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref file) = self.0.file {
            write!(f, "{}", file.display())?;
        }
        write!(f, ":{}:{}", self.0.end_row + 1, self.0.end_col + 1)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub file: Option<Arc<PathBuf>>,
    pub row: usize,
    pub col: usize,
    pub msg: Cow<'static, str>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref file) = self.file {
            write!(f, "{}", file.display())?;
        }
        write!(f, ":{}:{}: {}", self.row + 1, self.col + 1, self.msg.as_ref())
    }
}

impl Error for ParseError {

}


impl ParseError {
    pub fn with_cursor<I: Into<Cow<'static, str>>>(cursor: &Cursor, msg: I) -> Self {
        Self {
            file: cursor.file().clone(),
            row: cursor.row(),
            col: cursor.col(),
            msg: msg.into(),
        }
    }

    pub fn with_location<I: Into<Cow<'static, str>>>(location: &Location, msg: I) -> Self {
        Self {
            file: location.file.clone(),
            row: location.row,
            col: location.col,
            msg: msg.into(),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::{token::tokenizers, lexer::{Builder, Lexer}, cursor::Cursor};


    #[test]
    fn it_works() {
        let lexer: Lexer<(), ()> = 
            Builder::whitespace()
            .append(tokenizers::Comment)
            .append(tokenizers::QuotedString::<'"'>)
            .append(tokenizers::RawString)
            .append(tokenizers::Number)
            .append(tokenizers::Identifier)
            .append(tokenizers::Symbol(HashMap::from_iter([
                    ('(', 1), 
                    (')', 2),
                    ('!', 3),
                    (';', 4),
                    ('{', 5),
                    ('}', 6),
                    (',', 7),
                ])))
            .build();

        let input = r###"
            // hahaha
            fn 测试() {
                println!("hello, world {}", 12e-1);
                12. 12.0find
                r"abc"r##"ab"#123"##
            }
        "###;
        
        let mut cursor = Cursor::new(input, 0, 0, Default::default());

        while let Some(r) = lexer.tokenize(&mut cursor, &mut ()) {
            match r {
                Ok(token) => {
                    if token.kind > 0 {
                        println!("Token #{} at {}:{}", token.kind, token.location.row + 1, token.location.col + 1);
                        println!(" - {:?}", token.data);
                    }
                }
                Err(e) => {
                    println!("Error: {e}");
                    break;
                }
            }
        }
    }
}
