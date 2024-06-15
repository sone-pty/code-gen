use super::{TypeInfo, Value};

pub struct Custom {
    pub ty: TypeInfo,
    pub args: Vec<String>,
}

impl Value for Custom {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
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
}
