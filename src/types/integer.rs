use super::{TypeInfo, Value};

pub struct Bool {
    pub ty: TypeInfo,
    pub val: bool,
}

impl Value for Bool {
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
        self.ty == TypeInfo::Bool
    }
}

pub struct Int {
    pub ty: TypeInfo,
    pub val: i32,
}

impl Value for Int {
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
        self.ty == TypeInfo::Int
    }
}

pub struct UInt {
    pub ty: TypeInfo,
    pub val: u32,
}

impl Value for UInt {
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
        self.ty == TypeInfo::Uint
    }
}

pub struct Short {
    pub ty: TypeInfo,
    pub val: i16,
}

impl Value for Short {
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
        self.ty == TypeInfo::Short
    }
}

pub struct UShort {
    pub ty: TypeInfo,
    pub val: u16,
}

impl Value for UShort {
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
        self.ty == TypeInfo::UShort
    }
}

pub struct Byte {
    pub ty: TypeInfo,
    pub val: u8,
}

impl Value for Byte {
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
        self.ty == TypeInfo::Byte
    }
}

pub struct SByte {
    pub ty: TypeInfo,
    pub val: u32,
}

impl Value for SByte {
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
        self.ty == TypeInfo::SByte
    }
}

pub struct Float {
    pub ty: TypeInfo,
    pub val: f32,
}

impl Value for Float {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}f", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Float
    }
}

pub struct Double {
    pub ty: TypeInfo,
    pub val: f64,
}

impl Value for Double {
    fn ty(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream.write_fmt(format_args!("{}", self.ty))?;
        Ok(())
    }

    fn value(&self, stream: &mut dyn std::fmt::Write) -> Result<(), crate::error::Error> {
        stream
            .write_fmt(format_args!("{}d", self.val))
            .map_err(|e| e.into())
    }

    fn check(&self) -> bool {
        self.ty == TypeInfo::Double
    }
}
