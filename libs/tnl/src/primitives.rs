use std::{borrow::{Cow, Borrow}, hash};

use vnlex::Location;
use vnutil::io::{WriteExt as _, WriteTo as _, ReadExt as _};

use crate::{Value, ValueLoad, ValueType};



#[derive(Debug, Clone)]
pub struct Null (pub Location);

impl<'a> Value<'a> for Null {
    fn location(&self) -> &Location {
        &self.0
    }

    fn ty(&self) -> ValueType {
        ValueType::Null
    }

    fn clone_boxed(&self) -> Box<dyn Value<'a> + 'a> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.null(self)
    }

    fn save_to<'r>(&'r self, _: &mut crate::StringLibrary<'r>, _: &mut dyn std::io::Write) -> std::io::Result<()> {
        Ok(())
    }

    fn as_null_ref(&self) -> Option<&Null> {
        Some(self)
    }
}

impl ValueLoad<'_> for Null {
    fn load_from(_: &[&'_ str], _: &mut std::io::Cursor<&'_ [u8]>) -> std::io::Result<Self> {
        Ok(Self(Location::DEFAULT))
    }
}


#[derive(Debug, Clone)]
pub struct Boolean {
    pub location: Location,
    pub value: bool,
}

impl<'a> Value<'a> for Boolean {
    fn location(&self) -> &Location {
        &self.location
    }

    fn ty(&self) -> ValueType {
        ValueType::Bool
    }

    fn clone_boxed(&self) -> Box<dyn Value<'a> + 'a> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.bool(self)
    }

    fn save_to<'r>(&'r self, _: &mut crate::StringLibrary<'r>, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.value.write_to(w)
    }

    fn as_bool_ref(&self) -> Option<&Boolean> {
        Some(self)
    }
}

impl ValueLoad<'_> for Boolean {
    fn load_from(_: &[&'_ str], r: &mut std::io::Cursor<&'_ [u8]>) -> std::io::Result<Self> {
        Ok(Self {
            location: Location::DEFAULT,
            value: r.read_to()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Integer {
    pub location: Location,
    pub minus: bool,
    pub value: u64,
}

impl Integer {
    pub fn to_i8(&self) -> Option<i8> {
        if self.minus && self.value > 0x80 || self.value >= 0x80 {
            None
        } else if self.minus {
            Some((!self.value).wrapping_add(1) as i8)
        } else {
            Some(self.value as i8)
        }
    }

    pub fn to_u8(&self) -> Option<u8> {
        if self.minus || self.value > 0xFF {
            None 
        } else {
            Some(self.value as u8)
        }
    }

    pub fn to_i16(&self) -> Option<i16> {
        if self.minus && self.value > 0x8000 || self.value >= 0x8000 {
            None
        } else if self.minus {
            Some((!self.value).wrapping_add(1) as i16)
        } else {
            Some(self.value as i16)
        }
    }

    pub fn to_u16(&self) -> Option<u16> {
        if self.minus || self.value > 0xFFFF {
            None 
        } else {
            Some(self.value as u16)
        }
    }

    pub fn to_i32(&self) -> Option<i32> {
        if self.minus && self.value > 0x80000000 || self.value >= 0x80000000 {
            None
        } else if self.minus {
            Some((!self.value).wrapping_add(1) as i32)
        } else {
            Some(self.value as i32)
        }
    }

    pub fn to_u32(&self) -> Option<u32> {
        if !self.minus && self.value <= 0xFFFFFFFF {
            Some(self.value as _)
        } else {
            None
        }
    }

    pub fn to_i64(&self) -> Option<i64> {
        if self.minus && self.value > 0x80000000_00000000 || self.value >= 0x80000000_00000000 {
            None
        } else if self.minus {
            Some((!self.value).wrapping_add(1) as i64)
        } else {
            Some(self.value as i64)
        }
    }

    pub fn to_u64(&self) -> Option<u64> {
        if !self.minus {
            Some(self.value)
        } else {
            None
        }
    }
}

impl<'a> Value<'a> for Integer {
    fn location(&self) -> &Location {
        &self.location
    }

    fn ty(&self) -> ValueType {
        ValueType::Int
    }

    fn clone_boxed(&self) -> Box<dyn Value<'a> + 'a> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.int(self)
    }

    fn save_to<'r>(&'r self, _: &mut crate::StringLibrary<'r>, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.minus.write_to(w)?;
        w.write_compressed_u64(self.value)
    }

    fn as_int_ref(&self) -> Option<&Integer> {
        Some(self)
    }
}

impl ValueLoad<'_> for Integer {
    fn load_from(_: &[&'_ str], r: &mut std::io::Cursor<&'_ [u8]>) -> std::io::Result<Self> {
        Ok(Self {
            location: Location::DEFAULT,
            minus: r.read_to()?,
            value: r.read_compressed_u64()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Float {
    pub location: Location,
    pub value: f64,
}

impl<'a> Value<'a> for Float {
    fn location(&self) -> &Location {
        &self.location
    }

    fn ty(&self) -> ValueType {
        ValueType::Float
    }

    fn clone_boxed(&self) -> Box<dyn Value<'a> + 'a> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.float(self)
    }

    fn save_to<'r>(&'r self, _: &mut crate::StringLibrary<'r>, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.value.write_to(w)
    }

    fn as_float_ref(&self) -> Option<&Float> {
        Some(self)
    }
}

impl ValueLoad<'_> for Float {
    fn load_from(_: &[&'_ str], r: &mut std::io::Cursor<&'_ [u8]>) -> std::io::Result<Self> {
        Ok(Self {
            location: Location::DEFAULT,
            value: r.read_to()?,
        })
}
}

#[derive(Debug, Clone)]
pub struct String<'a> {
    pub location: Location,
    pub value: Cow<'a, str>,
}

impl<'a> Value<'a> for String<'a> {
    fn location(&self) -> &Location {
        &self.location
    }

    fn ty(&self) -> ValueType {
        ValueType::String
    }

    fn clone_boxed(&self) -> Box<dyn Value<'a> + 'a> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.string(self)
    }

    fn save_to<'r>(&'r self, strings: &mut crate::StringLibrary<'r>, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        w.write_compressed_usize(strings.get_index(self.value.as_ref()), usize::MAX)
    }

    fn as_string_ref(&self) -> Option<&String<'a>> {
        Some(self)
    }

    fn as_str(&self) -> Option<&str> {
        Some(&self.value)
    }
}

impl<'a> ValueLoad<'a> for String<'a> {
    fn load_from(strings: &[&'a str], r: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<Self> {
        Ok(Self {
            location: Location::DEFAULT,
            value: {
                let index = r.read_compressed_usize(strings.len())?;
                unsafe { *strings.get_unchecked(index) }.into()
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Ident<'a> {
    pub location: Location,
    pub value: Cow<'a, str>,
}

impl<'a> Value<'a> for Ident<'a> {
    fn location(&self) -> &Location {
        &self.location
    }

    fn ty(&self) -> ValueType {
        ValueType::Ident
    }

    fn clone_boxed(&self) -> Box<dyn Value<'a> + 'a> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.ident(self)
    }

    fn save_to<'r>(&'r self, strings: &mut crate::StringLibrary<'r>, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        w.write_compressed_usize(strings.get_index(self.value.as_ref()), usize::MAX)
    }

    fn as_ident_ref(&self) -> Option<&Ident<'a>> {
        Some(self)
    }

    fn as_str(&self) -> Option<&str> {
        Some(&self.value)
    }
}

impl<'a> ValueLoad<'a> for Ident<'a> {
    fn load_from(strings: &[&'a str], r: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<Self> {
        Ok(Self {
            location: Location::DEFAULT,
            value: {
                let index = r.read_compressed_usize(strings.len())?;
                unsafe { *strings.get_unchecked(index) }.into()
            }
        })
    }
}

impl hash::Hash for Ident<'_> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for Ident<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Ident<'_> {}

impl Borrow<str> for Ident<'_> {
    fn borrow(&self) -> &str {
        &self.value
    }
}