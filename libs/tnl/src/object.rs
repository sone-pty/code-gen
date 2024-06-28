use std::borrow::Cow;

use vnlex::{Location, ParseError};
use vnutil::io::{WriteExt as _, WriteTo, ReadExt as _};

use crate::{attributes::Attributes, Array, Boolean, Float, Ident, Integer, String, StringLibrary, Value, ValueLoad, ValueType};



#[derive(Clone, Debug)]
pub struct Object<'a> {
    pub base: Array<'a>,
    pub ns: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
    pub attributes: Attributes<'a>,
}

impl<'a> Value<'a> for Object<'a> {
    fn location(&self) -> &vnlex::Location {
        &self.base.location
    }

    fn ty(&self) -> ValueType {
        ValueType::Object
    }

    fn clone_boxed(&self) -> Box<dyn Value<'a> + 'a> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.object(self)
    }

    fn save_to<'r>(&'r self, strings: &mut crate::StringLibrary<'r>, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        w.write_compressed_usize(strings.get_index(self.name.as_ref()), usize::MAX)?;
        if let Some(ref ns) = self.ns {
            true.write_to(w)?;
            w.write_compressed_usize(strings.get_index(ns.as_ref()), usize::MAX)?;
        } else {
            false.write_to(w)?
        }
        w.write_compressed_usize(self.attributes.len(), usize::MAX)?;
        for (k, v) in self.attributes.iter() {
            k.save_to(strings, w)?;
            v.ty().write_to(w)?;
            v.save_to(strings, w)?;
        }
        self.base.save_to(strings, w)
    }

    fn as_object_ref(&self) -> Option<&Object<'a>> {
        Some(self)
    }
}

impl<'a> ValueLoad<'a> for Object<'a> {
    fn load_from(strings: &[&'a str], r: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<Self> {
        Ok(Self {
            name: {
                let index = r.read_compressed_usize(strings.len())?;
                unsafe { *strings.get_unchecked(index) }.into()
            },
            ns: if r.read_to()? {
                Some({
                    let index = r.read_compressed_usize(strings.len())?;
                    unsafe { *strings.get_unchecked(index) }.into()
                })
            } else {
                None
            },
            attributes: {
                let len = r.read_compressed_usize(usize::MAX)?;
                let mut attributes = Attributes::with_capacity(len);
                for _ in 0..len {
                    attributes.insert(ValueLoad::load_from(strings, r)?, ValueType::load_value(strings, r)?);
                }
                attributes
            },
            base: ValueLoad::load_from(strings, r)?,
        })
    }
}

impl<'a> Object<'a> {
    pub fn query_bool(&self, name: &str) -> Result<&Boolean, Option<&dyn Value<'a>>> {
        if let Some(t) = self.attributes.get(name) {
            match t.as_bool_ref() {
                Some(t) => Ok(t),
                _ => Err(Some(t.as_ref()))
            }
        } else {
            Err(None)
        }
    }

    pub fn query_int(&self, name: &str) -> Result<&Integer, Option<&dyn Value<'a>>> {
        if let Some(t) = self.attributes.get(name) {
            match t.as_int_ref() {
                Some(t) => Ok(t),
                _ => Err(Some(t.as_ref()))
            }
        } else {
            Err(None)
        }
    }

    pub fn query_float(&self, name: &str) -> Result<&Float, Option<&dyn Value<'a>>> {
        if let Some(t) = self.attributes.get(name) {
            match t.as_float_ref() {
                Some(t) => Ok(t),
                _ => Err(Some(t.as_ref()))
            }
        } else {
            Err(None)
        }
    }

    pub fn query_ident(&self, name: &str) -> Result<&Ident<'a>, Option<&dyn Value<'a>>> {
        if let Some(t) = self.attributes.get(name) {
            match t.as_ident_ref() {
                Some(t) => Ok(t),
                _ => Err(Some(t.as_ref()))
            }
        } else {
            Err(None)
        }
    }

    pub fn query_string(&self, name: &str) -> Result<&String<'a>, Option<&dyn Value<'a>>> {
        if let Some(t) = self.attributes.get(name) {
            match t.as_string_ref() {
                Some(t) => Ok(t),
                _ => Err(Some(t.as_ref()))
            }
        } else {
            Err(None)
        }
    }

    pub fn query_array(&self, name: &str) -> Result<&Array<'a>, Option<&dyn Value<'a>>> {
        if let Some(t) = self.attributes.get(name) {
            match t.as_array_ref() {
                Some(t) => Ok(t),
                _ => Err(Some(t.as_ref()))
            }
        } else {
            Err(None)
        }
    }

    pub fn query_object(&self, name: &str) -> Result<&Object<'a>, Option<&dyn Value<'a>>> {
        if let Some(t) = self.attributes.get(name) {
            match t.as_object_ref() {
                Some(t) => Ok(t),
                _ => Err(Some(t.as_ref()))
            }
        } else {
            Err(None)
        }
    }

    pub fn query_ident_or_string(&self, name: &str) -> Result<(&Location, &str), Option<&dyn Value<'a>>> {
        if let Some(t) = self.attributes.get(name) {
            match t.as_str() {
                Some(s) => Ok((t.location(), s)),
                _ => Err(Some(t.as_ref()))
            }
        } else {
            Err(None)
        }
    }

    pub fn load(data: &'a [u8]) -> Result<Self, Option<ParseError>> {
        if let Some((sign, data)) = data.split_at_checked(4) {
            if sign == b"TNL\0" {
                return Self::load_binary(data).map_err(|_| None)
            } 
        }
        crate::parse(unsafe { std::str::from_utf8_unchecked(data) }, 0, 0, None)
            .map_err(|e| Some(e))
    }

    pub fn save_binary<W: std::io::Write + ?Sized>(&self, w: &mut W) -> std::io::Result<()> {
        let mut lib = StringLibrary::new();
        let mut buf = Vec::<u8>::new();
        buf.write_compressed_usize(self.attributes.len(), usize::MAX)?;
        for (k, v) in self.attributes.iter() {
            k.save_to(&mut lib, &mut buf)?;
            v.ty().write_to(&mut buf)?;
            v.save_to(&mut lib, &mut buf)?;
        }
        self.base.save_to(&mut lib, &mut buf)?;
        w.write_all(b"TNL\0")?;
        lib.write_to(w)?;
        w.write_all(&buf)
    }

    pub fn load_binary(data: &'a [u8]) -> std::io::Result<Self> {
        let mut r = std::io::Cursor::new(data);
        let len = r.read_compressed_usize(usize::MAX - 1)?;
        let mut strings = Vec::<&'a str>::with_capacity(len);
        strings.push("");
        for _ in 0..len {
            let len = r.read_compressed_usize(usize::MAX)?;
            let start = r.position() as usize;
            let end = start + len;
            if let Some(t) = data.get(start..end) {
                strings.push(unsafe { std::str::from_utf8_unchecked(t) });
            } else {
                return Err(std::io::ErrorKind::InvalidData.into());
            }
            r.set_position(end as _);
        }
        let strings = strings.as_slice();
        let r = &mut r;
        Ok(Self {
            name: "".into(),
            ns: None,
            attributes: {
                let len = r.read_compressed_usize(usize::MAX)?;
                let mut attributes = Attributes::with_capacity(len);
                for _ in 0..len {
                    attributes.insert(ValueLoad::load_from(strings, r)?, ValueType::load_value(strings, r)?);
                }
                attributes
            },
            base: ValueLoad::load_from(strings, r)?,
        })
    }
}


impl<'a> TryFrom<&'a str> for Object<'a> {
    type Error = ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        crate::parser::parse(value, 0, 0, None)
    }
}