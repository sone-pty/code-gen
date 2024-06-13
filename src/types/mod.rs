use std::fmt::{Display, Write};

use crate::error::Error;

pub(crate) mod custom;
pub(crate) mod integer;
pub(crate) mod list;
pub(crate) mod string;

#[allow(dead_code)]
pub trait Value {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), Error>;
    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), Error>;
    fn check(&self) -> bool;
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq)]
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
    Enum,
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
            TypeInfo::Enum => todo!(),
            TypeInfo::String => f.write_str("string"),
            TypeInfo::LString => f.write_str("int"),
            TypeInfo::List(val) => f.write_fmt(format_args!("List<{}>", val)),
            TypeInfo::Tuple(vals) => {
                f.write_str("Tuple<")?;
                for v in &vals[0..vals.len() - 1] {
                    f.write_fmt(format_args!("{}, ", v))?;
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
                    f.write_fmt(format_args!("{}, ", v))?;
                }
                f.write_fmt(format_args!("{}", vals.last().unwrap()))?;
                f.write_char('>')
            }
            TypeInfo::ShortList => f.write_str("ShortList"),
            TypeInfo::Custom(ident) => f.write_str(&ident),
        }
    }
}
