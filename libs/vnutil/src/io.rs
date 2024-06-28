

use std::{io::{Read, Write, Result, ErrorKind, Seek}, mem::{MaybeUninit, size_of}, slice::{from_raw_parts_mut, from_raw_parts}};
use vncint::{try_decompress_u64, try_decompress_i64, CompressedInt, Compressing};

#[cfg(feature = "derive")]
pub use vnutil_derive::{WriteTo, ReadFrom};

pub trait ReadExt: Read {

    fn read_compressed_u64(&mut self) -> Result<u64> {
        try_decompress_u64(|buf| self.read_exact(buf))
    }

    fn read_compressed_i64(&mut self) -> Result<i64> {
        try_decompress_i64(|buf| self.read_exact(buf))
    }

    fn read_compressed_usize(&mut self, max: usize) -> Result<usize> {
        try_decompress_u64(|buf| self.read_exact(buf)).and_then(|len| {
            if len > max as u64 {
                Err(ErrorKind::InvalidData.into())
            } else {
                Ok(len as usize)
            }
        })
    }

    fn read_string(&mut self, max_bytes: usize) -> Result<String> {
        let len = self.read_compressed_usize(max_bytes)?;
        if len == 0 {
            Ok(String::new())
        } else {
            let mut vec = Vec::with_capacity(len);
            unsafe {
                vec.set_len(len);
            }
            self.read_exact(&mut vec)?;
            String::from_utf8(vec).map_err(|_| ErrorKind::InvalidData.into())
        }
    }

    unsafe fn read_string_unchecked(&mut self, max_bytes: usize) -> Result<String> {
        let len = self.read_compressed_usize(max_bytes)?;
        if len == 0 {
            Ok(String::new())
        } else {
            let mut vec = Vec::with_capacity(len);
            vec.set_len(len);
            self.read_exact(&mut vec)?;
            Ok(String::from_utf8_unchecked(vec))
        }
    }

    fn read_to<T: ReadFrom>(&mut self) -> Result<T> {
        T::read_from(self)
    }
}

impl<R: Read + ?Sized> ReadExt for R {}

pub trait WriteExt: Write {

    fn write_compressed_u64(&mut self, value: u64) -> Result<()> {
        value.compress(|buf| self.write_all(buf))
    }

    fn write_compressed_i64(&mut self, value: i64) -> Result<()> {
        value.compress(|buf| self.write_all(buf))
    }

    fn write_compressed_usize(&mut self, value: usize, max: usize) -> Result<()> {
        if value > max {
            return Err(ErrorKind::InvalidInput.into());
        }
        value.compress(|buf| self.write_all(buf))
    }

    fn write_string<A: AsRef<str>>(&mut self, max_bytes: usize, value: A) -> Result<()> {
        let value = value.as_ref();
        let bytes = value.as_bytes();
        self.write_compressed_usize(bytes.len(), max_bytes)?;
        self.write_all(bytes)
    }

}

impl<W: Write + ?Sized> WriteExt for W {}

pub trait ReadFrom: Sized {
    fn read_from<R: Read + ?Sized>(r: &mut R) -> Result<Self>;
}

pub trait WriteTo {
    fn write_to<W: Write + ?Sized>(&self, w: &mut W) -> Result<()>;
}

impl ReadFrom for bool {
    fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<bool> {
        match u8::read_from(s)? {
            0_u8 => Ok(false),
            1_u8 => Ok(true),
            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }
}

impl WriteTo for bool {
    fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
        if *self { 1u8 } else { 0u8 }.write_to(s)
    }
}

impl ReadFrom for String {
    fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<String> {
        s.read_string(usize::MAX)
    }
}

impl WriteTo for str {
    fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
        self.as_bytes().write_to(s)
    }
}

impl WriteTo for String {
    fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
        self.as_bytes().write_to(s)
    }
}

impl ReadFrom for usize {
    fn read_from<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        r.read_compressed_usize(usize::MAX)
    }
}

impl WriteTo for usize {
    fn write_to<W: Write + ?Sized>(&self, w: &mut W) -> Result<()> {
        w.write_compressed_usize(*self, usize::MAX)
    }
}

impl<T: ReadFrom> ReadFrom for Option<T> {
    fn read_from<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        Ok(if r.read_to()? {
            Some(r.read_to()?)
        } else {
            None
        })
    }
}

impl<T: WriteTo> WriteTo for Option<T> {
    fn write_to<W: Write + ?Sized>(&self, w: &mut W) -> Result<()> {
        if let Some(t) = self {
            true.write_to(w)?;
            t.write_to(w)
        } else {
            false.write_to(w)
        }
    }
}

impl<T: CompressedInt> ReadFrom for Compressing<T> {
    fn read_from<R: Read + ?Sized>(r: &mut R) -> Result<Self> {
        Ok(Compressing(T::decompress_from(r)?))
    }
}

impl<T: CompressedInt + Copy> WriteTo for Compressing<T> {
    fn write_to<W: Write + ?Sized>(&self, w: &mut W) -> Result<()> {
        self.0.compress_to(w)
    }
}

impl<T: ReadFrom, const N: usize> ReadFrom for Box<[T; N]> {
    default fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<Self> {
        let mut ret = MaybeUninit::uninit_array();
        for e in ret.iter_mut() {
            e.write(s.read_to()?);
        }
        Ok(Box::new(unsafe { MaybeUninit::array_assume_init(ret) }))
    }
}

impl<T: ReadFrom> ReadFrom for Box<[T]> {
    default fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<Self> {
        let len = s.read_compressed_usize(usize::MAX)?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read_from(s)?);
        }
        Ok(vec.into_boxed_slice())
    }
}

impl<T: ReadFrom> ReadFrom for Vec<T> {
    default fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<Self> {
        let len = s.read_compressed_usize(usize::MAX)?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read_from(s)?);
        }
        Ok(vec)
    }
}

impl<T: WriteTo, const N: usize> WriteTo for [T; N] {
    default fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
        for i in self {
            i.write_to(s)?;
        }
        Ok(())
    }
}

impl<T: WriteTo> WriteTo for [T] {
    default fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
        s.write_compressed_usize(self.len(), usize::MAX)?;
        for i in self {
            i.write_to(s)?;
        }
        Ok(())
    }
}

impl<T: WriteTo> WriteTo for Box<T> {
    fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
        self.as_ref().write_to(s)
    }
}

macro_rules! impl_copyable {
    ($($T:ty)*) => {$(
        impl ReadFrom for $T {
            fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<$T> {
                unsafe {
                    let mut ret: $T = MaybeUninit::zeroed().assume_init();
                    s.read_exact(from_raw_parts_mut(&mut ret as *mut $T as _, size_of::<$T>()))?;
                    Ok(ret)
                }
            }
        }
        impl<const N: usize> ReadFrom for Box<[$T; N]> {
            fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<Self> {
                unsafe {
                    let mut ret: Box<[$T; N]> = Box::new(MaybeUninit::zeroed().assume_init());
                    s.read_exact(from_raw_parts_mut(ret.as_mut_ptr() as _, size_of::<[$T; N]>()))?;
                    Ok(ret)
                }
            }
        }
        impl ReadFrom for Vec<$T> {
            fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<Self> {
                let len = s.read_to()?;
                let mut ret = Vec::with_capacity(len);
                unsafe {
                    ret.set_len(len);
                    s.read_exact(from_raw_parts_mut(ret.as_mut_ptr() as _, size_of::<$T>() * len))?;
                }
                Ok(ret)
            }
        }
        impl ReadFrom for Box<[$T]> {
            fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<Self> {
                let len = s.read_to()?;
                let mut ret = Vec::with_capacity(len);
                unsafe {
                    ret.set_len(len);
                    s.read_exact(from_raw_parts_mut(ret.as_mut_ptr() as _, size_of::<$T>() * len))?;
                }
                Ok(ret.into_boxed_slice())
            }
        }
        impl WriteTo for $T {
            fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
                s.write_all(unsafe { from_raw_parts(self as *const _ as _, size_of::<$T>()) })
            }
        }
        impl<const N: usize> WriteTo for [$T; N] {
            fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
                s.write_all(unsafe { from_raw_parts(self.as_ptr() as _, size_of::<[$T; N]>()) })
            }
        }
        impl WriteTo for [$T] {
            fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
                self.len().write_to(s)?;
                s.write_all(unsafe { from_raw_parts(self.as_ptr() as _, size_of::<$T>() * self.len()) })
            }
        }
    )*};
}

impl_copyable!{u8 i8 u16 i16 u32 i32 u64 i64 f32 f64}

macro_rules! impl_tuple {
    () => {
        impl ReadFrom for () {
            fn read_from<R: Read + ?Sized>(_: &mut R) -> Result<()> {
                Ok(())
            }
        }
        impl WriteTo for () {
            fn write_to<W: Write + ?Sized>(&self, _: &mut W) -> Result<()> {
                Ok(())
            }
        }

    };
    ($($name:ident)+) => {
        impl<$($name: ReadFrom),+> ReadFrom for ($($name,)+) {
            fn read_from<R: Read + ?Sized>(s: &mut R) -> Result<Self> {
                Ok(($($name::read_from(s)?,)+))
            }
        }
        impl<$($name: WriteTo),+> WriteTo for ($($name,)+) {
            #[allow(non_snake_case)]
            fn write_to<W: Write + ?Sized>(&self, s: &mut W) -> Result<()> {
                let ($(ref $name,)+) = *self;
                $($name.write_to(s)?;)+
                Ok(())
            }
        }
    };
}

impl_tuple! {}
impl_tuple! { A }
impl_tuple! { A B }
impl_tuple! { A B C }
impl_tuple! { A B C D }
impl_tuple! { A B C D E }
impl_tuple! { A B C D E F }
impl_tuple! { A B C D E F G }
impl_tuple! { A B C D E F G H }
impl_tuple! { A B C D E F G H I }
impl_tuple! { A B C D E F G H I J }
impl_tuple! { A B C D E F G H I J K }
impl_tuple! { A B C D E F G H I J K L }


pub trait ReadAndSeek: Read + Seek {}

impl<T: Read + Seek + ?Sized> ReadAndSeek for T {}
