use std::fmt::{Display, Write};

use crate::error::Error;

pub(crate) mod custom;
pub(crate) mod r#enum;
pub(crate) mod numbers;
pub(crate) mod sequence;
pub(crate) mod string;

#[allow(dead_code)]
pub trait Value {
    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), Error>;
    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), Error>;
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), Error>;
    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), Error>;
    fn check(&self) -> Result<(), Error>;
    fn ty_info(&self) -> &TypeInfo;
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeInfo {
    Int,
    Uint,
    Bool,
    Short,
    UShort,
    Float,
    Double,
    Decimal,
    Byte,
    SByte,
    Enum(String, String),
    String,
    LString,
    List(Box<TypeInfo>),
    Tuple(Vec<Box<TypeInfo>>),
    Array(Box<TypeInfo>),
    FixedArray(Box<TypeInfo>, usize),
    ValueTuple(Vec<Box<TypeInfo>>),
    ShortList,
    Custom(String),
}

impl TypeInfo {
    #[inline]
    pub fn is_string(&self) -> bool {
        match self {
            TypeInfo::String => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_enum(&self) -> bool {
        match self {
            TypeInfo::Enum(_, _) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_array_or_list(&self) -> bool {
        match self {
            Self::Array(_) | Self::FixedArray(_, _) | Self::List(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_lstring_or_lstringarr(&self) -> bool {
        self.is_lstring() || self.is_lstring_arr()
    }

    #[inline]
    pub fn is_lstring(&self) -> bool {
        self == &TypeInfo::LString
    }

    #[inline]
    pub fn is_lstring_arr(&self) -> bool {
        match self {
            TypeInfo::Array(e) if e.as_ref() == &TypeInfo::LString => true,
            TypeInfo::FixedArray(e, _) if e.as_ref() == &TypeInfo::LString => true,
            _ => false,
        }
    }

    #[inline]
    pub fn contains_string_or_lstring(&self) -> bool {
        match self {
            TypeInfo::String | TypeInfo::LString => true,
            TypeInfo::List(v) | TypeInfo::Array(v) | TypeInfo::FixedArray(v, _) => {
                v.as_ref() == &TypeInfo::String || v.as_ref() == &TypeInfo::LString
            }
            TypeInfo::Tuple(v) | TypeInfo::ValueTuple(v) => v
                .iter()
                .any(|v| v.as_ref() == &TypeInfo::String || v.as_ref() == &TypeInfo::LString),
            _ => false,
        }
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeInfo::Int => f.write_str("int"),
            TypeInfo::Uint => f.write_str("uint"),
            TypeInfo::Bool => f.write_str("bool"),
            TypeInfo::Short => f.write_str("short"),
            TypeInfo::UShort => f.write_str("ushort"),
            TypeInfo::Float => f.write_str("float"),
            TypeInfo::Double => f.write_str("double"),
            TypeInfo::Decimal => f.write_str("decimal"),
            TypeInfo::Byte => f.write_str("byte"),
            TypeInfo::SByte => f.write_str("sbyte"),
            TypeInfo::Enum(base, name) => f.write_fmt(format_args!("E{}{}", base, name)),
            TypeInfo::String => f.write_str("string"),
            TypeInfo::LString => f.write_str("int"),
            TypeInfo::List(val) => f.write_fmt(format_args!("List<{}>", val)),
            TypeInfo::Tuple(vals) => {
                f.write_str("Tuple<")?;
                for v in &vals[0..vals.len() - 1] {
                    f.write_fmt(format_args!("{},", v))?;
                }
                f.write_fmt(format_args!("{}", vals.last().unwrap()))?;
                f.write_char('>')
            }
            TypeInfo::Array(val) | TypeInfo::FixedArray(val, _) => {
                f.write_fmt(format_args!("{}[]", val))
            }
            TypeInfo::ValueTuple(vals) => {
                f.write_str("ValueTuple<")?;
                for v in &vals[0..vals.len() - 1] {
                    f.write_fmt(format_args!("{},", v))?;
                }
                f.write_fmt(format_args!("{}", vals.last().unwrap()))?;
                f.write_char('>')
            }
            TypeInfo::ShortList => f.write_str("ShortList"),
            TypeInfo::Custom(ident) => f.write_str(&ident),
        }
    }
}
