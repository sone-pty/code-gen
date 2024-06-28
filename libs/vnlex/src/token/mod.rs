use std::borrow::Cow;

use crate::{ParseError, cursor::Cursor, Location};

pub mod tokenizers;


#[derive(Debug)]
pub struct Token<'a, T> {
    pub kind: u32,
    pub content: &'a str,
    pub data: Data<'a, T>,
    pub location: Location,
}

impl<'a, T> Token<'a, T> {
    pub fn from_cursor(cursor: &Cursor<'a>, kind: u32, offset: usize, data: Data<'a, T>, row: usize, col: usize) -> Self {
        Token {
            kind,
            content: unsafe { cursor.sub_content(offset, cursor.offset() - offset) },
            data,
            location: cursor.location_from(row, col),
        }
    }
} 


#[derive(Debug)]
pub enum Data<'a, T> {
    None,
    Id(u32),
    Integer(u64),
    Float(f64),
    String(Cow<'a, str>),
    CodeBlock(usize, &'a str),
    Custom(T),
}

impl<'a, T> Data<'a, T> {
    pub fn is_none(&self) -> bool {
        match self {
            Data::None => true,
            _ => false,
        }
    }

    pub fn get_id(&self) -> Option<u32> {
        match self {
            Data::Id(t) => Some(*t),
            _ => None,
        }
    }

    pub fn get_integer(&self) -> Option<u64> {
        match self {
            Data::Integer(t) => Some(*t),
            _ => None,
        }
    }

    pub fn get_float(&self) -> Option<f64> {
        match self {
            Data::Float(t) => Some(*t),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<&Cow<'a, str>> {
        match self {
            Data::String(t) => Some(t),
            _ => None,
        }
    }

    pub fn into_string(self) -> Option<Cow<'a, str>> {
        match self {
            Data::String(t) => Some(t),
            _ => None,
        }
    }

    pub fn get_code_block(&self) -> Option<(usize, &'a str)> {
        match self {
            Data::CodeBlock(a, b) => Some((*a, *b)),
            _ => None,
        }
    }

    pub fn get_custom(&self) -> Option<&T> {
        match self {
            Data::Custom(t) => Some(t),
            _ => None,
        }
    }

    pub fn into_custom(self) -> Option<T> {
        match self {
            Data::Custom(t) => Some(t),
            _ => None,
        }
    }
}

pub trait Tokenizer<'a, T, M> {
    fn tokenize(&self, cursor: &mut Cursor<'a>, ctx: &mut M) -> Option<Result<Token<'a, T>, ParseError>>;
}

impl<'a, 'f, T, M, F> Tokenizer<'a, T, M> for F
where
    F: Fn(&mut Cursor<'a>, &mut M) -> Option<Result<Token<'a, T>, ParseError>>,
{
    fn tokenize(&self, cursor: &mut Cursor<'a>, ctx: &mut M) -> Option<Result<Token<'a, T>, ParseError>> {
        self(cursor, ctx)
    }
}