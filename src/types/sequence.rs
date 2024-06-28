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

    fn check(&self) -> Result<(), crate::error::Error> {
        for v in self.vals.iter() {
            v.check()?;
        }
        Ok(())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        for v in self.vals.iter() {
            v.check()?;
        }
        Ok(())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.is_null || self.vals.is_empty() {
            return Ok(());
        }
        
        let TypeInfo::FixedArray(ref sub, nums) = self.ty else {
            return Err("Check failed, expected fixed array type".into());
        };
        
        // TODO: reform
        if sub.is_string() && self.vals.len() == 1 {
            let mut t = String::new();
            unsafe { self.vals.get_unchecked(0).code_fmt(&mut t) }?;
            if t == "\"\"" {
                return Ok(());
            }
        }

        if nums == 0 || self.vals.len() != nums {
            return Err(format!("Fixed array does not match number of values, expected `{}`, actual nums = {}", nums, self.vals.len()).into());
        } else {
            for v in self.vals.iter() {
                v.check()?;
            }
            Ok(())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        let TypeInfo::Array(_) = self.ty else {
            return Err("Check failed, expected array type".into());
        };

        for v in self.vals.iter() {
            v.check()?;
        }
        Ok(())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        let TypeInfo::Tuple(ref types) = self.ty else {
            return Err("Check failed, expected tuple type".into());
        };

        if types.len() != self.vals.len() {
            return Err(format!("Tuple type {} does not match number of values, expected `{}`, actual nums = {}", self.ty, types.len(), self.vals.len()).into());
        } else {
            for v in self.vals.iter() {
                v.check()?;
            }
            Ok(())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        let TypeInfo::ValueTuple(ref types) = self.ty else {
            return Err("Check failed, expected valuetuple type".into());
        };

        if types.len() != self.vals.len() {
            return Err(format!("ValueTuple type {} does not match number of values, expected `{}`, actual nums = {}", self.ty, types.len(), self.vals.len()).into());
        } else {
            for v in self.vals.iter() {
                v.check()?;
            }
            Ok(())
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
