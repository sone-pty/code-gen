use super::{TypeInfo, Value};

pub struct SString {
    pub ty: TypeInfo,
    pub val: String,
}

impl Value for SString {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::String
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }
}

pub struct LString {
    pub ty: TypeInfo,
    #[allow(dead_code)]
    pub raw: String,
    pub idx: i32,
}

impl Value for LString {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.idx))?;
        Ok(())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::LString
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }
}
