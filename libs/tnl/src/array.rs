use vnlex::Location;
use vnutil::io::{WriteExt as _, WriteTo as _, ReadExt as _};

use crate::{Value, ValueLoad, ValueType};



#[derive(Debug)]
pub struct Array<'a> {
    pub location: Location,
    pub elements: Vec<Box<dyn Value<'a> + 'a>>,
}

impl Clone for Array<'_> {
    fn clone(&self) -> Self {
        Self {
            location: self.location.clone(),
            elements: self.elements.iter().map(|t| t.clone_boxed()).collect(),
        }
    }
}

impl<'a> Value<'a> for Array<'a> {
    fn location(&self) -> &Location {
        &self.location
    }

    fn ty(&self) -> crate::ValueType {
        ValueType::Array
    }

    fn clone_boxed(&self) -> Box<dyn Value<'a> + 'a> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.array(self)
    }

    fn save_to<'r>(&'r self, strings: &mut crate::StringLibrary<'r>, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        w.write_compressed_usize(self.elements.len(), usize::MAX)?;
        for e in self.elements.iter() {
            e.ty().write_to(w)?;
            e.save_to(strings, w)?;
        }
        Ok(())
    }

    fn as_array_ref(&self) -> Option<&Array<'a>> {
        Some(self)
    }
}

impl<'a> ValueLoad<'a> for Array<'a> {
    fn load_from(strings: &[&'a str], r: &mut std::io::Cursor<&'a [u8]>) -> std::io::Result<Self> {
        let len = r.read_compressed_usize(usize::MAX)?;
        let mut elements = Vec::with_capacity(len);
        for _ in 0..len {
            elements.push(ValueType::load_value(strings, r)?);
        }
        Ok(Self {
            location: Location::DEFAULT,
            elements,
        })
    }
}