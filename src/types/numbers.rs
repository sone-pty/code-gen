use super::{TypeInfo, Value};

pub struct Bool {
    pub ty: TypeInfo,
    pub val: bool,
}

impl Value for Bool {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Bool
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }
}

pub struct Int {
    pub ty: TypeInfo,
    pub val: i32,
}

impl Value for Int {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Int
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }
}

pub struct Decimal {
    pub ty: TypeInfo,
    pub val: rust_decimal::Decimal,
}

impl Value for Decimal {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}m", self.val))?;
        Ok(())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Decimal
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}m", self.val))?;
        Ok(())
    }
}

pub struct UInt {
    pub ty: TypeInfo,
    pub val: u32,
}

impl Value for UInt {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Uint
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }
}

pub struct Short {
    pub ty: TypeInfo,
    pub val: i16,
}

impl Value for Short {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Short
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }
}

pub struct UShort {
    pub ty: TypeInfo,
    pub val: u16,
}

impl Value for UShort {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::UShort
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }
}

pub struct Byte {
    pub ty: TypeInfo,
    pub val: u8,
}

impl Value for Byte {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Byte
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }
}

pub struct SByte {
    pub ty: TypeInfo,
    pub val: i8,
}

impl Value for SByte {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::SByte
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}", self.val))
            .map_err(|e| e.into())
    }
}

pub struct Float {
    pub ty: TypeInfo,
    pub val: f32,
}

impl Value for Float {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}f", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Float
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}f", self.val))
            .map_err(|e| e.into())
    }
}

pub struct Double {
    pub ty: TypeInfo,
    pub val: f64,
}

impl Value for Double {
    fn ty_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code_fmt(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}d", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Double
    }

    fn ty_info(&self) -> &TypeInfo {
        &self.ty
    }

    fn ty(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn code(&self, stream: &mut dyn std::io::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}d", self.val))
            .map_err(|e| e.into())
    }
}
