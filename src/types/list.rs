use super::{TypeInfo, Value};

pub struct List {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
}

impl Value for List {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("new {}{{", self.ty))?;
        if !self.vals.is_empty() {
            for v in &self.vals[0..self.vals.len() - 1] {
                v.value(stream)?;
                stream.write_str(", ")?;
            }
            self.vals.last().unwrap().value(stream)?;
        }
        stream.write_char('}')?;
        Ok(())
    }

    fn check(&self) -> bool {
        todo!()
    }
}

pub struct Array {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
}

impl Value for Array {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("new {}{{", self.ty))?;
        if !self.vals.is_empty() {
            for v in &self.vals[0..self.vals.len() - 1] {
                v.value(stream)?;
                stream.write_str(", ")?;
            }
            self.vals.last().unwrap().value(stream)?;
        }
        stream.write_char('}')?;
        Ok(())
    }

    fn check(&self) -> bool {
        todo!()
    }
}

pub struct Tuple {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
}

impl Value for Tuple {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("new {}{{", self.ty))?;
        if !self.vals.is_empty() {
            for v in &self.vals[0..self.vals.len() - 1] {
                v.value(stream)?;
                stream.write_str(", ")?;
            }
            self.vals.last().unwrap().value(stream)?;
        }
        stream.write_char('}')?;
        Ok(())
    }

    fn check(&self) -> bool {
        todo!()
    }
}

pub struct ValueTuple {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
}

impl Value for ValueTuple {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("new {}{{", self.ty))?;
        if !self.vals.is_empty() {
            for v in &self.vals[0..self.vals.len() - 1] {
                v.value(stream)?;
                stream.write_str(", ")?;
            }
            self.vals.last().unwrap().value(stream)?;
        }
        stream.write_char('}')?;
        Ok(())
    }

    fn check(&self) -> bool {
        todo!()
    }
}
