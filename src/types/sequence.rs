use super::{TypeInfo, Value};

pub struct List {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
    pub is_null: bool,
}

impl Value for List {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write_str("null")?;
        } else {
            stream.write_fmt(format_args!("new {}{{", self.ty))?;
            if !self.vals.is_empty() {
                for v in &self.vals[0..self.vals.len() - 1] {
                    v.code_fmt(stream)?;
                    stream.write_str(",")?;
                }
                self.vals.last().unwrap().code_fmt(stream)?;
            }
            stream.write_char('}')?;
        }
        Ok(())
    }

    fn check(&self) -> bool {
        self.vals.iter().all(|v| v.check())
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write("null".as_bytes())?;
        } else {
            stream.write_fmt(format_args!("new {}{{", self.ty))?;
            if !self.vals.is_empty() {
                for v in &self.vals[0..self.vals.len() - 1] {
                    v.code(stream)?;
                    stream.write(",".as_bytes())?;
                }
                self.vals.last().unwrap().code(stream)?;
            }
            stream.write("}".as_bytes())?;
        }
        Ok(())
    }
}

pub struct ShortList {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
    pub is_null: bool,
}

impl Value for ShortList {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write_str("null")?;
        } else {
            stream.write_fmt(format_args!("new {}(", self.ty))?;
            if !self.vals.is_empty() {
                for v in &self.vals[0..self.vals.len() - 1] {
                    v.code_fmt(stream)?;
                    stream.write_str(",")?;
                }
                self.vals.last().unwrap().code_fmt(stream)?;
            }
            stream.write_char(')')?;
        }
        Ok(())
    }

    fn check(&self) -> bool {
        self.vals.iter().all(|v| v.check())
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write("null".as_bytes())?;
        } else {
            stream.write_fmt(format_args!("new {}(", self.ty))?;
            if !self.vals.is_empty() {
                for v in &self.vals[0..self.vals.len() - 1] {
                    v.code(stream)?;
                    stream.write(",".as_bytes())?;
                }
                self.vals.last().unwrap().code(stream)?;
            }
            stream.write(")".as_bytes())?;
        }
        Ok(())
    }
}

pub struct FixedArray {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
    pub is_null: bool,
}

impl Value for FixedArray {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write_str("null")?;
        } else {
            stream.write_fmt(format_args!("new {}{{", self.ty))?;
            if !self.vals.is_empty() {
                for v in &self.vals[0..self.vals.len() - 1] {
                    v.code_fmt(stream)?;
                    stream.write_str(",")?;
                }
                self.vals.last().unwrap().code_fmt(stream)?;
            }
            stream.write_char('}')?;
        }
        Ok(())
    }

    fn check(&self) -> bool {
        let TypeInfo::FixedArray(_, nums) = self.ty else {
            return false;
        };

        if nums == 0 || self.vals.len() != nums {
            false
        } else {
            self.vals.iter().all(|v| v.check())
        }
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write("null".as_bytes())?;
        } else {
            stream.write_fmt(format_args!("new {}{{", self.ty))?;
            if !self.vals.is_empty() {
                for v in &self.vals[0..self.vals.len() - 1] {
                    v.code(stream)?;
                    stream.write(",".as_bytes())?;
                }
                self.vals.last().unwrap().code(stream)?;
            }
            stream.write("}".as_bytes())?;
        }
        Ok(())
    }
}

pub struct Array {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
    pub is_null: bool,
}

impl Value for Array {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write_str("null")?;
        } else {
            stream.write_fmt(format_args!("new {}{{", self.ty))?;
            if !self.vals.is_empty() {
                for v in &self.vals[0..self.vals.len() - 1] {
                    v.code_fmt(stream)?;
                    stream.write_str(",")?;
                }
                self.vals.last().unwrap().code_fmt(stream)?;
            }
            stream.write_char('}')?;
        }
        Ok(())
    }

    fn check(&self) -> bool {
        let TypeInfo::Array(_) = self.ty else {
            return false;
        };
        self.vals.iter().all(|v| v.check())
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write("null".as_bytes())?;
        } else {
            stream.write_fmt(format_args!("new {}{{", self.ty))?;
            if !self.vals.is_empty() {
                for v in &self.vals[0..self.vals.len() - 1] {
                    v.code(stream)?;
                    stream.write(",".as_bytes())?;
                }
                self.vals.last().unwrap().code(stream)?;
            }
            stream.write("}".as_bytes())?;
        }
        Ok(())
    }
}

pub struct Tuple {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
}

impl Value for Tuple {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("new {}(", self.ty))?;
        if !self.vals.is_empty() {
            for v in &self.vals[0..self.vals.len() - 1] {
                v.code_fmt(stream)?;
                stream.write_str(",")?;
            }
            self.vals.last().unwrap().code_fmt(stream)?;
        }
        stream.write_char(')')?;
        Ok(())
    }

    fn check(&self) -> bool {
        let TypeInfo::Tuple(ref types) = self.ty else {
            return false;
        };

        if types.len() != self.vals.len() {
            false
        } else {
            self.vals.iter().all(|v| v.check())
        }
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("new {}(", self.ty))?;
        if !self.vals.is_empty() {
            for v in &self.vals[0..self.vals.len() - 1] {
                v.code(stream)?;
                stream.write(",".as_bytes())?;
            }
            self.vals.last().unwrap().code(stream)?;
        }
        stream.write(")".as_bytes())?;
        Ok(())
    }
}

pub struct ValueTuple {
    pub ty: TypeInfo,
    pub vals: Vec<Box<dyn Value>>,
}

impl Value for ValueTuple {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("new {}(", self.ty))?;
        if !self.vals.is_empty() {
            for v in &self.vals[0..self.vals.len() - 1] {
                v.code_fmt(stream)?;
                stream.write_str(",")?;
            }
            self.vals.last().unwrap().code_fmt(stream)?;
        }
        stream.write_char(')')?;
        Ok(())
    }

    fn check(&self) -> bool {
        let TypeInfo::ValueTuple(ref types) = self.ty else {
            return false;
        };

        if types.len() != self.vals.len() {
            false
        } else {
            self.vals.iter().all(|v| v.check())
        }
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("new {}(", self.ty))?;
        if !self.vals.is_empty() {
            for v in &self.vals[0..self.vals.len() - 1] {
                v.code(stream)?;
                stream.write(",".as_bytes())?;
            }
            self.vals.last().unwrap().code(stream)?;
        }
        stream.write(")".as_bytes())?;
        Ok(())
    }
}
