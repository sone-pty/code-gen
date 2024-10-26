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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::Bool {
            Ok(())
        } else {
            Err("Check failed, expected bool type".into())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::Int {
            Ok(())
        } else {
            Err("Check failed, expected int type".into())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::Decimal {
            Ok(())
        } else {
            Err("Check failed, expected decimal type".into())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::Uint {
            Ok(())
        } else {
            Err("Check failed, expected uint type".into())
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
        if self.val == i16::MIN {
            stream.write_str("short.MinValue").map_err(|e| e.into())
        } else {
            stream
                .write_fmt(format_args!("{}", self.val))
                .map_err(|e| e.into())
        }
    }

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::Short {
            Ok(())
        } else {
            Err("Check failed, expected short type".into())
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
        if self.val == i16::MIN {
            stream.write("short.MinValue".as_bytes())?;
            Ok(())
        } else {
            stream
                .write_fmt(format_args!("{}", self.val))
                .map_err(|e| e.into())
        }
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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::UShort {
            Ok(())
        } else {
            Err("Check failed, expected ushort type".into())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::Byte {
            Ok(())
        } else {
            Err("Check failed, expected byte type".into())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::SByte {
            Ok(())
        } else {
            Err("Check failed, expected sbyte type".into())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::Float {
            Ok(())
        } else {
            Err("Check failed, expected float type".into())
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

    fn check(&self) -> Result<(), crate::error::Error> {
        if self.ty == TypeInfo::Double {
            Ok(())
        } else {
            Err("Check failed, expected double type".into())
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
        stream
            .write_fmt(format_args!("{}d", self.val))
            .map_err(|e| e.into())
    }
}
