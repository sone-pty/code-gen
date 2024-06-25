use super::{TypeInfo, Value};

pub struct Custom {
    pub ty: TypeInfo,
    pub args: Vec<String>,
    pub is_null: bool,
}

impl Value for Custom {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write_str("null")?;
        } else {
            stream.write_fmt(format_args!("new {}(", self.ty))?;

            if !self.args.is_empty() {
                for v in (&self.args[0..self.args.len() - 1])
                    .iter()
                    .map(|v| v.as_str().trim())
                {
                    if v == "{}" {
                        stream.write_str("default, ")?;
                    } else if v.starts_with("{") {
                        stream.write_fmt(format_args!("new []{}, ", v))?;
                    } else {
                        stream.write_fmt(format_args!("{}, ", v))?;
                    }
                }

                let v = self.args.last().unwrap().as_str().trim();
                if v == "{}" {
                    stream.write_str("default")?;
                } else if v.starts_with("{") {
                    stream.write_fmt(format_args!("new []{}", v))?;
                } else {
                    stream.write_fmt(format_args!("{}", v))?;
                }
            }
            stream.write_char(')')?;
        }
        Ok(())
    }

    fn check(&self) -> bool {
        let TypeInfo::Custom(_) = self.ty else {
            return false;
        };
        true
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

            if !self.args.is_empty() {
                for v in (&self.args[0..self.args.len() - 1])
                    .iter()
                    .map(|v| v.as_str().trim())
                {
                    if v == "{}" {
                        stream.write("default, ".as_bytes())?;
                    } else if v.starts_with("{") {
                        stream.write_fmt(format_args!("new []{}, ", v))?;
                    } else {
                        stream.write_fmt(format_args!("{}, ", v))?;
                    }
                }

                let v = self.args.last().unwrap().as_str().trim();
                if v == "{}" {
                    stream.write("default".as_bytes())?;
                } else if v.starts_with("{") {
                    stream.write_fmt(format_args!("new []{}", v))?;
                } else {
                    stream.write_fmt(format_args!("{}", v))?;
                }
            }
            stream.write(")".as_bytes())?;
        }
        Ok(())
    }
}
