#![feature(str_from_raw_parts)]

use std::{fmt, io, path::PathBuf, sync::Arc};

use vnlex::cursor::Cursor;


mod primitives;
mod array;
mod object;
mod states;
mod parser;
mod from_tnl;
mod accessor;
mod string_library;
mod builder;
pub mod attributes;
pub mod export;
pub mod format;

pub use primitives::*;
pub use array::*;
pub use object::*;
pub use parser::parse;
pub use vnlex::{Location, ParseError};
pub use from_tnl::FromTnl;
pub use string_library::StringLibrary;
pub use accessor::{Accessor, ArrayAccessor, ObjectAccessor, AccessError, AccessErrorKind};
pub use builder::Builder;

use vnutil::io::{ReadFrom, WriteTo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Null,
    Bool,
    Int,
    Float,
    Ident,
    String,
    Array,
    Object,
}

impl WriteTo for ValueType {
    fn write_to<W: io::Write + ?Sized>(&self, w: &mut W) -> io::Result<()> {
        let val: u8 = match self {
            ValueType::Null => 0,
            ValueType::Bool => 1,
            ValueType::Int => 2,
            ValueType::Float => 3,
            ValueType::Ident => 4,
            ValueType::String => 5,
            ValueType::Array => 6,
            ValueType::Object => 7,
        };
        val.write_to(w)
    }
}

impl ReadFrom for ValueType {
    fn read_from<R: io::Read + ?Sized>(r: &mut R) -> io::Result<Self> {
        Ok(match u8::read_from(r)? {
            0 => Self::Null,
            1 => Self::Bool,
            2 => Self::Int,
            3 => Self::Float,
            4 => Self::Ident,
            5 => Self::String,
            6 => Self::Array,
            7 => Self::Object,
            _ => return Err(io::ErrorKind::InvalidData.into())
        })
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Null => f.write_str("`null`"),
            ValueType::Bool => f.write_str("{bool}"),
            ValueType::Int => f.write_str("{int}"),
            ValueType::Float => f.write_str("{float}"),
            ValueType::Ident => f.write_str("{ident}"),
            ValueType::String => f.write_str("{string}"),
            ValueType::Array => f.write_str("{array}"),
            ValueType::Object => f.write_str("{object}"),
        }
    }
}

impl ValueType {
    pub fn load_value<'a>(strings: &[&'a str], r: &mut io::Cursor<&'a [u8]>) -> io::Result<Box<dyn Value<'a> + 'a>> {
        Ok(match u8::read_from(r)? {
            0 => Box::new(Null::load_from(strings, r)?),
            1 => Box::new(Boolean::load_from(strings, r)?),
            2 => Box::new(Integer::load_from(strings, r)?),
            3 => Box::new(Float::load_from(strings, r)?),
            4 => Box::new(Ident::load_from(strings, r)?),
            5 => Box::new(String::load_from(strings, r)?),
            6 => Box::new(Array::load_from(strings, r)?),
            7 => Box::new(Object::load_from(strings, r)?),
            _ => return Err(io::ErrorKind::InvalidData.into())
        })
    }
}

pub trait Visitor<'a> {
    fn object(&mut self, val: &Object<'a>);
    fn array(&mut self, val: &Array<'a>);
    fn null(&mut self, val: &Null);
    fn bool(&mut self, val: &Boolean);
    fn int(&mut self, val: &Integer);
    fn float(&mut self, val: &Float);
    fn string(&mut self, val: &String<'a>);
    fn ident(&mut self, val: &Ident<'a>);
}

pub trait Value<'a>: fmt::Debug {
    fn location(&self) -> &Location;
    fn ty(&self) -> ValueType;
    fn clone_boxed(&self) -> Box<dyn Value<'a> + 'a>;
    fn accept(&self, visitor: &mut dyn Visitor<'a>);
    fn save_to<'r>(&'r self, strings: &mut StringLibrary<'r>, w: &mut dyn io::Write) -> io::Result<()>;

    fn as_null_ref(&self) -> Option<&Null> { None }
    fn as_bool_ref(&self) -> Option<&Boolean> { None }
    fn as_int_ref(&self) -> Option<&Integer> { None }
    fn as_float_ref(&self) -> Option<&Float> { None }
    fn as_ident_ref(&self) -> Option<&Ident<'a>> { None }
    fn as_string_ref(&self) -> Option<&String<'a>> { None }
    fn as_array_ref(&self) -> Option<&Array<'a>> { None }
    fn as_object_ref(&self) -> Option<&Object<'a>> { None }
    fn as_str(&self) -> Option<&str> { None }
}

pub trait ValueLoad<'a>: Sized {
    fn load_from(strings: &[&'a str], r: &mut io::Cursor<&'a [u8]>) -> io::Result<Self>;
}

pub fn parse_value(input: &str, row: usize, col: usize, file: Option<Arc<PathBuf>>) -> Result<Option<Box<dyn Value + '_>>, ParseError> {
    let parser = &*crate::parser::PARSER;
    let mut cursor = Cursor::new(input, row, col, file);
    let root = parser.syntaxer
        .parse_optional(parser.lexer.tokenizing(&mut cursor, &mut ()))
        .map_err(|t| t.into(&cursor))?
        ;

    if let Some(root) = root {
        Ok(Some(crate::parser::parse_value(root)?))
    } else {
        Ok(None)
    }
}

impl<'a> TryFrom<&'a str> for Box<dyn Value<'a> + 'a> {
    type Error = ParseError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let parser = &*crate::parser::PARSER;
        let mut cursor = Cursor::new(value, 0, 0, None);
        let root = parser.syntaxer
            .parse(parser.lexer.tokenizing(&mut cursor, &mut ()))
            .map_err(|t| t.into(&cursor))?
            ;

        crate::parser::parse_value(root)
    }
}

pub fn parse_to<T: FromTnl<Err = ParseError>>(content: &str) -> Result<T, ParseError> {
    let val: Box<dyn Value> = content.try_into()?;
    val.parse()
}

pub fn parse_object_to<T: FromTnl<Err = ParseError>>(content: &str) -> Result<T, ParseError> {
    let obj = parse(content, 0, 0, None)?;
    (&obj as &dyn Value).parse()
}

impl dyn Value<'_> + '_ {
    pub fn parse<T: FromTnl>(&self) -> Result<T, T::Err> {
        T::from_tnl(self)
    }
}


#[cfg(test)]
mod tests {
    use crate::{parse, Object};


    #[test]
    fn parse_object() {
        let obj = parse(r#"test r"file.ext""#, 0, 0, None).unwrap();
        assert_eq!(obj.base.elements[0].as_ident_ref().unwrap().value.as_ref(), "test");
        assert_eq!(obj.base.elements[1].as_str().unwrap(), "file.ext");
    }

    #[test]
    fn parse_value() {
        let value = crate::parse_value("1_0010", 0, 0, None).unwrap().unwrap();
        assert_eq!(value.as_int_ref().unwrap().to_u32(), Some(10010));
    }

    #[test]
    fn test_convert_bin() {
        let tnl = std::fs::read("test.tnl").unwrap();
        match Object::load(tnl.as_slice()) {
            Ok(v) => {
                let id = v.query_string("id").unwrap();
                println!("{}", id.value);
                let mut out = std::fs::File::options().create(true).truncate(true).write(true).open("test_bin").unwrap();
                let _ = v.save_binary(&mut out);
            }
            Err(e) => println!("{:?}", e)
        };
    }
}