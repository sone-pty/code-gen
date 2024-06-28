use super::{TypeInfo, Value};

pub struct Enum {
    pub ty: TypeInfo,
    pub ident: String,
    pub is_null: bool,
}

impl Value for Enum {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        if self.is_null {
            stream.write_str("null")?;
        } else {
            self.ty_fmt(stream)?;
            stream.write_fmt(format_args!(".{}", self.ident))?;
        }
        Ok(())
    }

    fn check(&self) -> Result<(), crate::error::Error> {
        let TypeInfo::Enum(_, _) = self.ty else {
            return Err("Check failed, expected enum type".into());
        };
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
            self.ty(stream)?;
            stream.write_fmt(format_args!(".{}", self.ident))?;
        }
        Ok(())
    }
}
