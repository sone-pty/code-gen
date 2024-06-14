use super::{TypeInfo, Value};

pub struct Enum {
    pub ty: TypeInfo,
    pub ident: String,
}

impl Value for Enum {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        self.ty(stream)?;
        stream.write_fmt(format_args!(".{}", self.ident))?;
        Ok(())
    }

    fn check(&self) -> bool {
        true
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }
}
