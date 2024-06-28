use std::{error::Error, fmt};

use vnlex::Location;

use crate::{Array, Object, Value, ValueType};

#[derive(Debug)]
pub struct AccessError<'a> {
    pub location: &'a Location,
    pub kind: AccessErrorKind,
}

#[derive(Debug)]
pub enum AccessErrorKind {
    WrongType { expect: ValueType, found: ValueType },
    WrongType2 { expect: (ValueType, ValueType), found: ValueType },
    OutOfRangeFor(&'static str),
    IndexOutOfRange(usize, usize),
    AttributeNotFound(String),
}

impl fmt::Display for AccessError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref file) = self.location.file {
            write!(f, "{}", file.display())?;
        }
        if self.location.row == self.location.end_row && self.location.col == self.location.end_col {
            write!(f, ":{}:{}", self.location.row + 1, self.location.col + 1)?;
        } else {
            write!(f, ":{}:{}-{}:{}", self.location.row + 1, self.location.col + 1, self.location.end_row + 1, self.location.end_col + 1)?;
        }
        write!(f, ": {}", self.kind)
    }
}

impl fmt::Display for AccessErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessErrorKind::WrongType { expect, found } => {
                write!(f, "expect {expect}, found {found}")
            }
            AccessErrorKind::WrongType2 { expect: (t1, t2), found } => {
                write!(f, "expect {t1} or {t2}, found {found}")
            }
            AccessErrorKind::OutOfRangeFor(ty) => {
                write!(f, "out of range for `{ty}`")
            }
            AccessErrorKind::IndexOutOfRange(index, len) => {
                write!(f, "index({index}) out of range(0..{len})")
            }
            AccessErrorKind::AttributeNotFound(name) => {
                write!(f, "attribute `{name}` not found")
            }
        }
    }
}

impl Error for AccessError<'_> {}

#[derive(Clone, Copy)]
pub struct Accessor<'r, 'a>(pub &'r dyn Value<'a>);

#[derive(Clone, Copy)]
pub struct ArrayAccessor<'r, 'a>(pub &'r Array<'a>);

#[derive(Clone, Copy)]
pub struct ObjectAccessor<'r, 'a>(pub &'r Object<'a>);

macro_rules! impl_as_ints {
    ($($func_name:ident $T:ident $impl_func:ident)*) => {
        impl<'r, 'a> Accessor<'r, 'a> {
        $(
            pub fn $func_name(self) -> Result<$T, AccessError<'r>> {
                if let Some(t) = self.0.as_int_ref() {
                    if let Some(t) = t.$impl_func() {
                        Ok(t)
                    } else {
                        Err(AccessError {
                            location: self.0.location(),
                            kind: AccessErrorKind::OutOfRangeFor(stringify!($T)),
                        })
                    }
                } else {
                    Err(AccessError {
                        location: self.0.location(),
                        kind: AccessErrorKind::WrongType { expect: ValueType::Int, found: self.0.ty() }
                    })
                }
            }
        )*
        }
    };
}

impl_as_ints! {
    as_i8 i8 to_i8
    as_u8 u8 to_u8
    as_i16 i16 to_i16
    as_u16 u16 to_u16
    as_i32 i32 to_i32
    as_u32 u32 to_u32
    as_i64 i64 to_i64
    as_u64 u64 to_u64
}

impl<'r, 'a> Accessor<'r, 'a> {
    pub fn is_null(self) -> bool {
        self.0.as_null_ref().is_some()
    }

    pub fn as_bool(self) -> Result<bool, AccessError<'r>> {
        if let Some(t) = self.0.as_bool_ref() {
            Ok(t.value)
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::WrongType { expect: ValueType::Bool, found: self.0.ty() }
            })
        }
    }

    pub fn as_f32(self) -> Result<f32, AccessError<'r>> {
        if let Some(t) = self.0.as_float_ref() {
            Ok(t.value as _)
        } else if let Some(t) = self.0.as_int_ref() {
            Ok(if t.minus { -(t.value as f32) } else { t.value as f32 })
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::WrongType2 { expect: (ValueType::Int, ValueType::Float), found: self.0.ty() }
            })
        }
    }

    pub fn as_f64(self) -> Result<f64, AccessError<'r>> {
        if let Some(t) = self.0.as_float_ref() {
            Ok(t.value)
        } else if let Some(t) = self.0.as_int_ref() {
            Ok(if t.minus { -(t.value as f64) } else { t.value as f64 })
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::WrongType2 { expect: (ValueType::Int, ValueType::Float), found: self.0.ty() }
            })
        }
    }

    pub fn as_ident(self) -> Result<&'r str, AccessError<'r>> {
        if let Some(t) = self.0.as_ident_ref() {
            Ok(t.value.as_ref())
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::WrongType { expect: ValueType::Ident, found: self.0.ty() }
            })
        }
    }

    pub fn as_str(self) -> Result<&'r str, AccessError<'r>> {
        if let Some(t) = self.0.as_str() {
            Ok(t)
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::WrongType2 { expect: (ValueType::String, ValueType::Ident), found: self.0.ty() }
            })
        }
    }

    pub fn as_array(self) -> Result<ArrayAccessor<'r, 'a>, AccessError<'r>> {
        if let Some(t) = self.0.as_array_ref() {
            Ok(ArrayAccessor(t))
        } else if let Some(t) = self.0.as_object_ref() {
            Ok(ArrayAccessor(&t.base))
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::WrongType2 { expect: (ValueType::Array, ValueType::Object), found: self.0.ty() }
            })
        }
    }

    pub fn as_object(self) -> Result<ObjectAccessor<'r, 'a>, AccessError<'r>> {
        if let Some(t) = self.0.as_object_ref() {
            Ok(ObjectAccessor(t))
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::WrongType { expect: ValueType::Object, found: self.0.ty() }
            })
        }
    }
}

impl<'r, 'a> ArrayAccessor<'r, 'a> {
    pub fn index(self, index: usize) -> Result<Accessor<'r, 'a>, AccessError<'r>> {
        if let Some(t) = self.0.elements.get(index) {
            Ok(Accessor(t.as_ref()))
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::IndexOutOfRange(index, self.0.elements.len()),
            })
        }
    }
}

impl<'r, 'a> ObjectAccessor<'r, 'a> {
    pub fn index(self, index: usize) -> Result<Accessor<'r, 'a>, AccessError<'r>> {
        if let Some(t) = self.0.base.elements.get(index) {
            Ok(Accessor(t.as_ref()))
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::IndexOutOfRange(index, self.0.base.elements.len()),
            })
        }
    }

    pub fn attribute(self, name: &str) -> Result<Accessor<'r, 'a>, AccessError<'r>> {
        if let Some(t) = self.0.attributes.get(name) {
            Ok(Accessor(t.as_ref()))
        } else {
            Err(AccessError {
                location: self.0.location(),
                kind: AccessErrorKind::AttributeNotFound(name.to_owned()),
            })
        }
    }

    pub fn optional_attribute(self, name: &str) -> Option<Accessor<'r, 'a>> {
        self.0.attributes.get(name).map(|t| Accessor(t.as_ref()))
    }
}