use super::{TypeInfo, Value};

pub struct Enum {
    pub ty: TypeInfo,
    pub ident: String,
}

impl Value for Enum {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        self.ty_fmt(stream)?;
        stream.write_fmt(format_args!(".{}", self.ident))?;
        Ok(())
    }

    fn check(&self) -> bool {
        let TypeInfo::Enum(_, _) = self.ty else {
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
        self.ty(stream)?;
        stream.write_fmt(format_args!(".{}", self.ident))?;
        Ok(())
    }
}
