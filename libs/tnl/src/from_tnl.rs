use vnlex::ParseError;

use crate::Value;



pub trait FromTnl: Sized {
    type Err;
    fn from_tnl(val: &dyn Value) -> Result<Self, Self::Err>;
}



impl FromTnl for bool {
    type Err = ParseError;

    fn from_tnl(val: &dyn Value) -> Result<Self, Self::Err> {
        if let Some(t) = val.as_bool_ref() {
            Ok(t.value)
        } else {
            Err(ParseError::with_location(val.location(), format!("expect {{bool}}, found {}", val.ty())))
        }
    }
}

macro_rules! impl_ints {
    ($($T:ty)*) => {$(
        impl FromTnl for $T {
            type Err = ParseError;
            fn from_tnl(val: &dyn Value) -> Result<Self, Self::Err> {
                if let Some(t) = val.as_int_ref() {
                    const MAX: u64 = <$T>::MAX as u64 + 1;
                    if t.minus && t.value > MAX || t.value >= MAX {
                        Err(ParseError::with_location(val.location(), "out of range"))
                    } else if t.minus {
                        Ok((!t.value).wrapping_add(1) as $T)
                    } else {
                        Ok(t.value as $T)
                    }
                } else {
                    Err(ParseError::with_location(val.location(), format!("expect {{int}}, found {}", val.ty())))
                }
            }
        }
    )*};
}

macro_rules! impl_uints {
    ($($T:ty)*) => {$(
        impl FromTnl for $T {
            type Err = ParseError;
            fn from_tnl(val: &dyn Value) -> Result<Self, Self::Err> {
                if let Some(t) = val.as_int_ref() {
                    if t.minus || t.value >= <$T>::MAX as _ {
                        Err(ParseError::with_location(val.location(), "out of range"))
                    } else {
                        Ok(t.value as $T)
                    }
                } else {
                    Err(ParseError::with_location(val.location(), format!("expect {{int}}, found {}", val.ty())))
                }
            }
        }
    )*};
}

impl_ints!{ i8 i16 i32 i64 isize }
impl_uints!{ u8 u16 u32 u64 usize }

macro_rules! impl_floats {
    ($($T:ty)*) => {$(
        impl FromTnl for $T {
            type Err = ParseError;
            fn from_tnl(val: &dyn Value) -> Result<Self, Self::Err> {
                if let Some(t) = val.as_float_ref() {
                    Ok(t.value as $T)
                } else if let Some(t) = val.as_int_ref() {
                    if t.minus {
                        Ok(-(t.value as $T))
                    } else {
                        Ok(t.value as $T)
                    }
                } else {
                    Err(ParseError::with_location(val.location(), format!("expect {{int}} for {{float}}, found {}", val.ty())))
                }
            }
        }
    )*};
}

impl_floats!{ f32 f64 }

impl<T: FromTnl> FromTnl for Option<T> {
    type Err = T::Err;
    fn from_tnl(val: &dyn Value) -> Result<Self, Self::Err> {
        if val.as_null_ref().is_some() {
            Ok(None)
        } else {
            Ok(Some(val.parse()?))
        }
    }
}

impl FromTnl for String {
    type Err = ParseError;
    fn from_tnl(val: &dyn Value) -> Result<Self, Self::Err> {
        if let Some(t) = val.as_str() {
            Ok(t.to_owned())
        } else {
            Err(ParseError::with_location(val.location(), format!("expect {{string}} for {{ident}}, found {}", val.ty())))
        }
    }
}

impl<T: FromTnl<Err = ParseError>> FromTnl for Vec<T> {
    type Err = ParseError;
    fn from_tnl(val: &dyn Value) -> Result<Self, Self::Err> {
        if let Some(arr) = val.as_array_ref() {
            let mut r = Vec::with_capacity(arr.elements.len());
            for t in arr.elements.iter() {
                r.push(t.parse()?);
            }
            Ok(r)
        } else if let Some(obj) = val.as_object_ref() {
            let mut r = Vec::with_capacity(obj.base.elements.len());
            for t in obj.base.elements.iter() {
                r.push(t.parse()?);
            }
            Ok(r)
        } else {
            Err(ParseError::with_location(val.location(), format!("expect {{array}} for {{object}}, found {}", val.ty())))
        }
    }
}