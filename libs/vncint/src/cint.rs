use std::io::{self, Write, Read};
use std::mem::MaybeUninit;
use std::{slice, fmt};
use std::iter::FusedIterator;
use std::marker::PhantomData;

pub enum DecompressError<E> {
    PosOverflow,
    NegOverflow,
    Read(E),
}

impl<E: fmt::Debug> fmt::Debug for DecompressError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PosOverflow => f.write_str("PosOverflow"),
            Self::NegOverflow => f.write_str("NegOverflow"),
            Self::Read(e) => e.fmt(f),
        }
    }
}

impl From<DecompressError<io::Error>> for io::Error {
    fn from(value: DecompressError<io::Error>) -> Self {
        match value {
            DecompressError::PosOverflow => Self::new(io::ErrorKind::InvalidData, "number too large to fit in target type"),
            DecompressError::NegOverflow => Self::new(io::ErrorKind::InvalidData, "number too small to fit in target type"),
            DecompressError::Read(e) => e,
        }
    }
}

impl<E> From<E> for DecompressError<E> {
    fn from(value: E) -> Self {
        DecompressError::Read(value)
    }
}

pub trait CompressedInt {
    fn decompress_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(lead: u8, f: F) -> Result<Self, DecompressError<E>> where Self: Sized;
    fn compress<T, F: FnMut(&[u8]) -> T>(self, f: F) -> T;

    fn decompress<E, F: FnMut(&mut [u8]) -> Result<(), E>>(mut f: F) -> Result<Self, DecompressError<E>> where Self: Sized {
        let mut lead = 0_u8;
        f(slice::from_mut(&mut lead))?;
        Self::decompress_with_lead(lead, f)
    }

    fn decompress_from<R: Read + ?Sized>(r: &mut R) -> io::Result<Self> where Self: Sized {
        Ok(Self::decompress(|buf| r.read_exact(buf))?)
    }

    fn decompress_with_lead_from<R: Read + ?Sized>(lead: u8, r: &mut R) -> io::Result<Self> where Self: Sized {
        Ok(Self::decompress_with_lead(lead, |buf| r.read_exact(buf))?)
    }

    fn compress_to<W: Write + ?Sized>(self, w: &mut W) -> io::Result<()> where Self: Sized {
        self.compress(|buf| w.write_all(buf))
    }
}

impl CompressedInt for u16 {
    fn decompress_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(lead: u8, f: F) -> Result<Self, DecompressError<E>> where Self: Sized {
        let n = try_decompress_u64_with_lead(lead, f)?;
        if n > u16::MAX as _ {
           Err(DecompressError::PosOverflow)
        } else {
            Ok(n as _)
        }
    }

    fn compress<T, F: FnMut(&[u8]) -> T>(self, f: F) -> T {
        compress_u64_with(self.into(), f)
    }
}

impl CompressedInt for u32 {
    fn decompress_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(lead: u8, f: F) -> Result<Self, DecompressError<E>> where Self: Sized {
        let n = try_decompress_u64_with_lead(lead, f)?;
        if n > u32::MAX as _ {
           Err(DecompressError::PosOverflow)
        } else {
            Ok(n as _)
        }
    }

    fn compress<T, F: FnMut(&[u8]) -> T>(self, f: F) -> T {
        compress_u64_with(self.into(), f)
    }
}

impl CompressedInt for usize {
    fn decompress_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(lead: u8, f: F) -> Result<Self, DecompressError<E>> where Self: Sized {
        let n = try_decompress_u64_with_lead(lead, f)?;
        if n > usize::MAX as _ {
           Err(DecompressError::PosOverflow)
        } else {
            Ok(n as _)
        }
    }

    fn compress<T, F: FnMut(&[u8]) -> T>(self, f: F) -> T {
        compress_u64_with(self as u64, f)
    }
}

impl CompressedInt for u64 {
    fn decompress_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(lead: u8, f: F) -> Result<Self, DecompressError<E>> where Self: Sized {
        Ok(try_decompress_u64_with_lead(lead, f)?)
    }

    fn compress<T, F: FnMut(&[u8]) -> T>(self, f: F) -> T {
        compress_u64_with(self, f)
    }
}

impl CompressedInt for i16 {
    fn decompress_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(lead: u8, f: F) -> Result<Self, DecompressError<E>> where Self: Sized {
        let n = try_decompress_i64_with_lead(lead, f)?;
        if n < i16::MIN as _ {
            Err(DecompressError::NegOverflow)
        } else if n > i16::MAX as _ {
            Err(DecompressError::PosOverflow)
        } else {
            Ok(n as _)
        }
    }

    fn compress<T, F: FnMut(&[u8]) -> T>(self, f: F) -> T {
        compress_i64_with(self.into(), f)
    }
}

impl CompressedInt for i32 {
    fn decompress_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(lead: u8, f: F) -> Result<Self, DecompressError<E>> where Self: Sized {
        let n = try_decompress_i64_with_lead(lead, f)?;
        if n < i32::MIN as _ {
            Err(DecompressError::NegOverflow)
        } else if n > i32::MAX as _ {
            Err(DecompressError::PosOverflow)
        } else {
            Ok(n as _)
        }
    }

    fn compress<T, F: FnMut(&[u8]) -> T>(self, f: F) -> T {
        compress_i64_with(self.into(), f)
    }
}

impl CompressedInt for isize {
    fn decompress_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(lead: u8, f: F) -> Result<Self, DecompressError<E>> where Self: Sized {
        let n = try_decompress_i64_with_lead(lead, f)?;
        if n < isize::MIN as _ {
            Err(DecompressError::NegOverflow)
        } else if n > isize::MAX as _ {
            Err(DecompressError::PosOverflow)
        } else {
            Ok(n as _)
        }
    }

    fn compress<T, F: FnMut(&[u8]) -> T>(self, f: F) -> T {
        compress_i64_with(self as i64, f)
    }
}

impl CompressedInt for i64 {
    fn decompress_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(lead: u8, f: F) -> Result<Self, DecompressError<E>> where Self: Sized {
        Ok(try_decompress_i64_with_lead(lead, f)?)
    }

    fn compress<T, F: FnMut(&[u8]) -> T>(self, f: F) -> T {
        compress_i64_with(self, f)
    }
}

pub fn get_compressed_data_len_by_lead(val: u8) -> usize {
    match val {
        n if n & 0x80 == 0 => 1,
        n if n & 0xC0 == 0x80 => 2,
        n if n & 0xE0 == 0xC0 => 3,
        n if n & 0xF0 == 0xE0 => 4,
        n if n & 0xF8 == 0xF0 => 5,
        n if n & 0xFC != 0xF8 => 6,
        n if n & 0xFE != 0xFC => 7,
        n if n == 0xFE => 8,
        _ => 9,
    }
}

pub fn compress_u64_with<T, F: FnMut(&[u8]) -> T>(val: u64, mut f: F) -> T {
    let mut data = val.to_be();
    let p = &mut data as *mut _ as *mut u8;
    let buf: &mut [u8];
    unsafe {
        match val {
            n if n <= 0x7F => {
                buf = slice::from_raw_parts_mut(p.offset(7), 1);
            }
            n if n <= 0x3FFF => {
                buf = slice::from_raw_parts_mut(p.offset(6), 2);
                buf[0] |= 0x80;
            }
            n if n <= 0x001F_FFFF => {
                buf = slice::from_raw_parts_mut(p.offset(5), 3);
                buf[0] |= 0xC0;
            }
            n if n <= 0x0FFF_FFFF => {
                buf = slice::from_raw_parts_mut(p.offset(4), 4);
                buf[0] |= 0xE0;
            }
            n if n <= 0x0007_FFFF_FFFF => {
                buf = slice::from_raw_parts_mut(p.offset(3), 5);
                buf[0] |= 0xF0;
            }
            n if n <= 0x03FF_FFFF_FFFF => {
                buf = slice::from_raw_parts_mut(p.offset(2), 6);
                buf[0] |= 0xF8;
            }
            n if n <= 0x0001_FFFF_FFFF_FFFF => {
                buf = slice::from_raw_parts_mut(p.offset(1), 7);
                buf[0] |= 0xFC;
            }
            n if n <= 0x00FF_FFFF_FFFF_FFFF => {
                buf = slice::from_raw_parts_mut(p, 8);
                buf[0] = 0xFE;
            }
            _ => {
                f(slice::from_ref(&0xFFu8));
                buf = slice::from_raw_parts_mut(p, 8);
            }
        }
    }
    f(buf)
}

pub fn compress_u64(val: u64, buf: &mut [u8; 9]) -> usize {
    let data = val.to_be();
    let p = &data as *const _ as *const u8;
    unsafe {
        match val {
            n if n <= 0x7F => {
                std::ptr::copy_nonoverlapping(p.add(7), buf.as_mut_ptr().add(8), 1);
                8
            }
            n if n <= 0x3FFF => {
                std::ptr::copy_nonoverlapping(p.add(6), buf.as_mut_ptr().add(7), 2);
                let lead = buf.get_unchecked_mut(7);
                *lead |= 0x80;
                7
            }
            n if n <= 0x001F_FFFF => {
                std::ptr::copy_nonoverlapping(p.add(5), buf.as_mut_ptr().add(6), 3);
                let lead = buf.get_unchecked_mut(6);
                *lead |= 0xC0;
                6
            }
            n if n <= 0x0FFF_FFFF => {
                std::ptr::copy_nonoverlapping(p.add(4), buf.as_mut_ptr().add(5), 4);
                let lead = buf.get_unchecked_mut(5);
                *lead |= 0xE0;
                5
            }
            n if n <= 0x0007_FFFF_FFFF => {
                std::ptr::copy_nonoverlapping(p.add(3), buf.as_mut_ptr().add(4), 5);
                let lead = buf.get_unchecked_mut(4);
                *lead |= 0xF0;
                4
            }
            n if n <= 0x03FF_FFFF_FFFF => {
                std::ptr::copy_nonoverlapping(p.add(2), buf.as_mut_ptr().add(3), 6);
                let lead = buf.get_unchecked_mut(3);
                *lead |= 0xF8;
                3
            }
            n if n <= 0x0001_FFFF_FFFF_FFFF => {
                std::ptr::copy_nonoverlapping(p.add(1), buf.as_mut_ptr().add(2), 7);
                let lead = buf.get_unchecked_mut(2);
                *lead |= 0xFC;
                2
            }
            n if n <= 0x00FF_FFFF_FFFF_FFFF => {
                std::ptr::copy_nonoverlapping(p.add(1), buf.as_mut_ptr().add(2), 7);
                let lead = buf.get_unchecked_mut(1);
                *lead = 0xFE;
                1
            }
            _ => {
                std::ptr::copy_nonoverlapping(p, buf.as_mut_ptr().add(1), 8);
                let lead = buf.get_unchecked_mut(8);
                *lead = 0xFF;
                0
            }
        }
    }
}


pub struct CompressU64Iter<'a> {
    data: u64,
    step: fn(&mut CompressU64Iter<'a>) -> Option<&'a [u8]>,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> CompressU64Iter<'a> {
    pub fn new(value: u64) -> CompressU64Iter<'a> {
        CompressU64Iter {
            data: value,
            step: Self::s1,
            phantom: PhantomData,
        }
    }

    const HEAD: u8 = 0xFFu8;

    fn s1(&mut self) -> Option<&'a [u8]> {
        unsafe {
            let val = self.data;
            self.data = val.to_be();
            let p = &mut self.data as *mut _ as *mut u8;
            match val {
                n if n <= 0x7F => {
                    self.step = Self::done;
                    Some(slice::from_raw_parts(p.offset(7), 1))
                }
                n if n <= 0x3FFF => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(6), 2);
                    buf[0] |= 0x80;
                    Some(buf)
                }
                n if n <= 0x001F_FFFF => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(5), 3);
                    buf[0] |= 0xC0;
                    Some(buf)
                }
                n if n <= 0x0FFF_FFFF => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(4), 4);
                    buf[0] |= 0xE0;
                    Some(buf)
                }
                n if n <= 0x0007_FFFF_FFFF => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(3), 5);
                    buf[0] |= 0xF0;
                    Some(buf)
                }
                n if n <= 0x03FF_FFFF_FFFF => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(2), 6);
                    buf[0] |= 0xF8;
                    Some(buf)
                }
                n if n <= 0x0001_FFFF_FFFF_FFFF => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(1), 7);
                    buf[0] |= 0xFC;
                    Some(buf)
                }
                n if n <= 0x00FF_FFFF_FFFF_FFFF => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p, 8);
                    buf[0] = 0xFE;
                    Some(buf)
                }
                _ => {
                    self.step = Self::s2;
                    Some(slice::from_ref(&Self::HEAD))
                }
            }
        }
    }

    fn s2(&mut self) -> Option<&'a [u8]> {
        unsafe {
            self.step = Self::done;
            Some(slice::from_raw_parts(&self.data as *const _ as *const _, 8))
        }
    }

    fn done(&mut self) -> Option<&'a [u8]> {
        None
    }
}

impl<'a> Iterator for CompressU64Iter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        (self.step)(self)
    }
}

impl FusedIterator for CompressU64Iter<'_> {}


pub fn decompress_u64_with<F: FnMut(&mut [u8])>(mut f: F) -> u64 {
    unsafe {
        let mut lead = MaybeUninit::uninit();
        f(slice::from_mut(lead.assume_init_mut()));
        decompress_u64_with_lead(lead.assume_init(), f)
    }
}

pub fn decompress_u64_with_lead<F: FnMut(&mut [u8])>(lead: u8, mut f: F) -> u64 {
    let mut data = 0u64;
    let p = &mut data as *mut _ as *mut u8;
    unsafe {
        match lead {
            n if n & 0x80 == 0 => {
                *p.offset(7) = n & 0x7F;
            }
            n if n & 0xC0 == 0x80 => {
                *p.offset(6) = n & 0x3F;
                f(slice::from_raw_parts_mut(p.offset(7), 1));
            }
            n if n & 0xE0 == 0xC0 => {
                *p.offset(5) = n & 0x1F;
                f(slice::from_raw_parts_mut(p.offset(6), 2));
            }
            n if n & 0xF0 == 0xE0 => {
                *p.offset(4) = n & 0x0F;
                f(slice::from_raw_parts_mut(p.offset(5), 3));
            }
            n if n & 0xF8 == 0xF0 => {
                *p.offset(3) = n & 0x07;
                f(slice::from_raw_parts_mut(p.offset(4), 4));
            }
            n if n & 0xFC != 0xF8 => {
                *p.offset(2) = n & 0x03;
                f(slice::from_raw_parts_mut(p.offset(3), 5));
            }
            n if n & 0xFE != 0xFC => {
                *p.offset(1) = n & 0x01;
                f(slice::from_raw_parts_mut(p.offset(2), 6));
            }
            n if n == 0xFE => {
                f(slice::from_raw_parts_mut(p.offset(1), 7));
            }
            _ => {
                f(slice::from_raw_parts_mut(p, 8));
            }
        }
    }
    u64::from_be(data)
}

pub fn decompress_u64(lead: u8) -> (usize, unsafe fn(u8, *const u8) -> u64) {
    unsafe {
        match lead {
            n if n & 0x80 == 0 => {
                (0, |lead, _| (lead & 0x7F) as u64)
            }
            n if n & 0xC0 == 0x80 => {
                (1, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(6) = lead & 0x3F;
                    std::ptr::copy_nonoverlapping(ptr, p.add(7), 1);
                    u64::from_be(data)
                })
            }
            n if n & 0xE0 == 0xC0 => {
                (2, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(5) = lead & 0x1F;
                    std::ptr::copy_nonoverlapping(ptr, p.add(6), 2);
                    u64::from_be(data)
                })
            }
            n if n & 0xF0 == 0xE0 => {
                (3, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(4) = lead & 0x0F;
                    std::ptr::copy_nonoverlapping(ptr, p.add(5), 3);
                    u64::from_be(data)
                })
            }
            n if n & 0xF8 == 0xF0 => {
                (4, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(3) = lead & 0x07;
                    std::ptr::copy_nonoverlapping(ptr, p.add(4), 4);
                    u64::from_be(data)
                })
            }
            n if n & 0xFC != 0xF8 => {
                (5, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(2) = lead & 0x03;
                    std::ptr::copy_nonoverlapping(ptr, p.add(3), 5);
                    u64::from_be(data)
                })
            }
            n if n & 0xFE != 0xFC => {
                (6, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(1) = lead & 0x01;
                    std::ptr::copy_nonoverlapping(ptr, p.add(2), 6);
                    u64::from_be(data)
                })
            }
            n if n == 0xFE => {
                (7, |_, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    std::ptr::copy_nonoverlapping(ptr, p.add(1), 7);
                    u64::from_be(data)
                })
            }
            _ => {
                (8, |_, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    std::ptr::copy_nonoverlapping(ptr, p, 8);
                    u64::from_be(data)
                })
            }
        }
    }
}


pub struct DecompressU64Iter<'a> {
    data: &'a mut u64,
    step: fn(&mut Self) -> Option<&'a mut [u8]>,
}

impl<'a> DecompressU64Iter<'a> {
    pub fn new(target: &'a mut u64) -> DecompressU64Iter<'a> {
        DecompressU64Iter {
            data: target,
            step: Self::s1,
        }
    }
    fn s1(&mut self) -> Option<&'a mut [u8]> {
        *self.data = 0;
        self.step = Self::s2;
        unsafe {
            let p = self.data as *mut _ as *mut u8;
            Some(slice::from_raw_parts_mut(p.offset(7), 1))
        }
    }
    fn s2(&mut self) -> Option<&'a mut [u8]> {
        unsafe {
            let p = self.data as *mut _ as *mut u8;
            match *p.offset(7) {
                n if n & 0x80 == 0 => {
                    self.last()
                }
                n if n & 0xC0 == 0x80 => {
                    *p.offset(6) = n & 0x3F;
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(7), 1))
                }
                n if n & 0xE0 == 0xC0 => {
                    *p.offset(5) = n & 0x1F;
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(6), 2))
                }
                n if n & 0xF0 == 0xE0 => {
                    *p.offset(4) = n & 0x0F;
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(5), 3))
                }
                n if n & 0xF8 == 0xF0 => {
                    *p.offset(3) = n & 0x07;
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(4), 4))
                }
                n if n & 0xFC != 0xF8 => {
                    *p.offset(2) = n & 0x03;
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(3), 5))
                }
                n if n & 0xFE != 0xFC => {
                    *p.offset(1) = n & 0x01;
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(2), 6))
                }
                n if n == 0xFE => {
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(1), 7))
                }
                _ => {
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p, 8))
                }
            }
        }
    }

    fn last(&mut self) -> Option<&'a mut [u8]> {
        *self.data = u64::from_be(*self.data);
        self.step = Self::done;
        None
    }

    fn done(&mut self) -> Option<&'a mut [u8]> {
        None
    }
}

impl<'a> Iterator for DecompressU64Iter<'a> {
    type Item = &'a mut [u8];
    fn next(&mut self) -> Option<&'a mut [u8]> {
        (self.step)(self)
    }
}

impl FusedIterator for DecompressU64Iter<'_> {}


pub fn compress_i64_with<T, F: FnMut(&[u8]) -> T>(val: i64, mut f: F) -> T {
    let mut data = val.to_be();
    let p = &mut data as *mut _ as *mut u8;
    let buf: &mut [u8];
    unsafe {
        match val {
            n if n <= 0x3F && n > -0x40 => {
                buf = slice::from_raw_parts_mut(p.offset(7), 1);
                buf[0] &= 0x7F;
            }
            n if n <= 0x1FFF && n > -0x2000 => {
                buf = slice::from_raw_parts_mut(p.offset(6), 2);
                buf[0] = (buf[0] & 0x3F) | 0x80;
            }
            n if n <= 0x000F_FFFF && n > -0x0010_0000 => {
                buf = slice::from_raw_parts_mut(p.offset(5), 3);
                buf[0] = (buf[0] & 0x1F) | 0xC0;
            }
            n if n <= 0x07FF_FFFF && n > -0x0800_0000 => {
                buf = slice::from_raw_parts_mut(p.offset(4), 4);
                buf[0] = (buf[0] & 0x0F) | 0xE0;
            }
            n if n <= 0x0003_FFFF_FFFF && n > -0x0004_0000_0000 => {
                buf = slice::from_raw_parts_mut(p.offset(3), 5);
                buf[0] = (buf[0] & 0x07) | 0xF0;
            }
            n if n <= 0x01FF_FFFF_FFFF && n > -0x0200_0000_0000 => {
                buf = slice::from_raw_parts_mut(p.offset(2), 6);
                buf[0] = (buf[0] & 0x03) | 0xF8;
            }
            n if n <= 0xFFFF_FFFF_FFFF && n > -0x0001_0000_0000_0000 => {
                buf = slice::from_raw_parts_mut(p.offset(1), 7);
                buf[0] = (buf[0] & 0x01) | 0xFC;
            }
            n if n <= 0x007F_FFFF_FFFF_FFFF && n > -0x0080_0000_0000_0000 => {
                buf = slice::from_raw_parts_mut(p, 8);
                buf[0] = 0xFE;
            }
            _ => {
                f(slice::from_ref(&0xFFu8));
                buf = slice::from_raw_parts_mut(p, 8);
            }
        }
    }
    f(buf)
}

pub fn compress_i64(val: i64, buf: &mut [u8; 9]) -> usize {
    let data = val.to_be();
    let p = &data as *const _ as *const u8;
    unsafe {
        match val {
            n if n <= 0x3F && n > -0x40 => {
                std::ptr::copy_nonoverlapping(p.add(7), buf.as_mut_ptr().add(8), 1);
                *buf.get_unchecked_mut(8) &= 0x7F;
                8
            }
            n if n <= 0x1FFF && n > -0x2000 => {
                std::ptr::copy_nonoverlapping(p.add(6), buf.as_mut_ptr().add(7), 2);
                let lead = buf.get_unchecked_mut(7);
                *lead = (*lead & 0x3F) | 0x80;
                7
            }
            n if n <= 0x000F_FFFF && n > -0x0010_0000 => {
                std::ptr::copy_nonoverlapping(p.add(5), buf.as_mut_ptr().add(6), 3);
                let lead = buf.get_unchecked_mut(6);
                *lead = (*lead & 0x1F) | 0xC0;
                6
            }
            n if n <= 0x07FF_FFFF && n > -0x0800_0000 => {
                std::ptr::copy_nonoverlapping(p.add(4), buf.as_mut_ptr().add(5), 4);
                let lead = buf.get_unchecked_mut(5);
                *lead = (*lead & 0x0F) | 0xE0;
                5
            }
            n if n <= 0x0003_FFFF_FFFF && n > -0x0004_0000_0000 => {
                std::ptr::copy_nonoverlapping(p.add(3), buf.as_mut_ptr().add(4), 5);
                let lead = buf.get_unchecked_mut(4);
                *lead = (*lead & 0x07) | 0xF0;
                4
            }
            n if n <= 0x01FF_FFFF_FFFF && n > -0x0200_0000_0000 => {
                std::ptr::copy_nonoverlapping(p.add(2), buf.as_mut_ptr().add(3), 6);
                let lead = buf.get_unchecked_mut(3);
                *lead = (*lead & 0x03) | 0xF8;
                3
            }
            n if n <= 0xFFFF_FFFF_FFFF && n > -0x0001_0000_0000_0000 => {
                std::ptr::copy_nonoverlapping(p.add(1), buf.as_mut_ptr().add(2), 7);
                let lead = buf.get_unchecked_mut(2);
                *lead = (*lead & 0x01) | 0xFC;
                2
            }
            n if n <= 0x007F_FFFF_FFFF_FFFF && n > -0x0080_0000_0000_0000 => {
                std::ptr::copy_nonoverlapping(p.add(1), buf.as_mut_ptr().add(2), 7);
                *buf.get_unchecked_mut(1) = 0xFE;
                1
            }
            _ => {
                std::ptr::copy_nonoverlapping(p, buf.as_mut_ptr().add(1), 8);
                *buf.get_unchecked_mut(0) = 0xFF;
                0
            }
        }
    }
}

pub struct CompressI64Iter<'a> {
    data: u64,
    step: fn(&mut CompressI64Iter<'a>) -> Option<&'a [u8]>,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> CompressI64Iter<'a> {
    pub fn new(value: i64) -> CompressI64Iter<'a> {
        CompressI64Iter {
            data: value as u64,
            step: Self::s1,
            phantom: PhantomData,
        }
    }

    const HEAD: u8 = 0xFFu8;

    fn s1(&mut self) -> Option<&'a [u8]> {
        unsafe {
            let val = self.data as i64;
            self.data = self.data.to_be();
            let p = &mut self.data as *mut _ as *mut u8;
            match val {
                n if n <= 0x3F && n > -0x40 => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(7), 1);
                    buf[0] &= 0x7F;
                    Some(buf)
                }
                n if n <= 0x1FFF && n > -0x2000 => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(6), 2);
                    buf[0] = (buf[0] & 0x3F) | 0x80;
                    Some(buf)
                }
                n if n <= 0x000F_FFFF && n > -0x0010_0000 => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(5), 3);
                    buf[0] = (buf[0] & 0x1F) | 0xC0;
                    Some(buf)
                }
                n if n <= 0x07FF_FFFF && n > -0x0800_0000 => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(4), 4);
                    buf[0] = (buf[0] & 0x0F) | 0xE0;
                    Some(buf)
                }
                n if n <= 0x0003_FFFF_FFFF && n > -0x0004_0000_0000 => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(3), 5);
                    buf[0] = (buf[0] & 0x07) | 0xF0;
                    Some(buf)
                }
                n if n <= 0x01FF_FFFF_FFFF && n > -0x0200_0000_0000 => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(2), 6);
                    buf[0] = (buf[0] & 0x03) | 0xF8;
                    Some(buf)
                }
                n if n <= 0xFFFF_FFFF_FFFF && n > -0x0001_0000_0000_0000 => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p.offset(1), 7);
                    buf[0] = (buf[0] & 0x01) | 0xFC;
                    Some(buf)
                }
                n if n <= 0x007F_FFFF_FFFF_FFFF && n > -0x0080_0000_0000_0000 => {
                    self.step = Self::done;
                    let buf = slice::from_raw_parts_mut(p, 8);
                    buf[0] = 0xFE;
                    Some(buf)
                }
                _ => {
                    self.step = Self::s2;
                    Some(slice::from_ref(&Self::HEAD))
                }
            }
        }
    }

    fn s2(&mut self) -> Option<&'a [u8]> {
        unsafe {
            self.step = Self::done;
            Some(slice::from_raw_parts(&self.data as *const _ as *const _, 8))
        }
    }

    fn done(&mut self) -> Option<&'a [u8]> {
        None
    }
}

impl<'a> Iterator for CompressI64Iter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        (self.step)(self)
    }
}

impl FusedIterator for CompressI64Iter<'_> {}



pub fn decompress_i64_with<F: FnMut(&mut [u8])>(mut f: F) -> i64 {
    unsafe {
        let mut lead = MaybeUninit::uninit();
        f(slice::from_mut(lead.assume_init_mut()));
        decompress_i64_with_lead(lead.assume_init(), f)
    }
}

pub fn decompress_i64_with_lead<F: FnMut(&mut [u8])>(lead: u8, mut f: F) -> i64 {
    let mut data = 0u64;
    let p = &mut data as *mut _ as *mut u8;
    unsafe {
        match lead {
            n if n & 0x80 == 0 => {
                *p.offset(7) = n & 0x7F;
                if n & (1 << 6) != 0 {
                    data |= 0xFFFF_FFFF_FFFF_FF80u64.to_be();
                }
            }
            n if n & 0xC0 == 0x80 => {
                *p.offset(6) = n & 0x3F;
                f(slice::from_raw_parts_mut(p.offset(7), 1));
                if n & (1 << 5) != 0 {
                    data |= 0xFFFF_FFFF_FFFF_C000u64.to_be();
                }
            }
            n if n & 0xE0 == 0xC0 => {
                *p.offset(5) = n & 0x1F;
                f(slice::from_raw_parts_mut(p.offset(6), 2));
                if n & (1 << 4) != 0 {
                    data |= 0xFFFF_FFFF_FFE0_0000u64.to_be();
                }
            }
            n if n & 0xF0 == 0xE0 => {
                *p.offset(4) = n & 0x0F;
                f(slice::from_raw_parts_mut(p.offset(5), 3));
                if n & (1 << 3) != 0 {
                    data |= !0xFFFF_FFFF_F000_0000u64.to_be();
                }
            }
            n if n & 0xF8 == 0xF0 => {
                *p.offset(3) = n & 0x07;
                f(slice::from_raw_parts_mut(p.offset(4), 4));
                if n & (1 << 2) != 0 {
                    data |= 0xFFFF_FFF8_0000_0000u64.to_be();
                }
            }
            n if n & 0xFC != 0xF8 => {
                *p.offset(2) = n & 0x03;
                f(slice::from_raw_parts_mut(p.offset(3), 5));
                if n & (1 << 1) != 0 {
                    data |= 0xFFFF_FC00_0000_0000u64.to_be();
                }
            }
            n if n & 0xFE != 0xFC => {
                *p.offset(1) = n & 0x01;
                f(slice::from_raw_parts_mut(p.offset(2), 6));
                if n & 1 != 0 {
                    data |= 0xFFFE_0000_0000_0000u64.to_be();
                }
            }
            n if n == 0xFE => {
                f(slice::from_raw_parts_mut(p.offset(1), 7));
                if data & 0x0080_0000_0000_0000u64.to_be() != 0 {
                    *p = 0xFF;
                }
            }
            _ => {
                f(slice::from_raw_parts_mut(p, 8));
            }
        }
    }
    i64::from_be(data as i64)
}

pub fn decompress_i64(lead: u8) -> (usize, unsafe fn(u8, *const u8) -> i64) {

    unsafe {
        match lead {
            n if n & 0x80 == 0 => {
                (0, |lead, _| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(7) = lead & 0x7F;
                    if lead & (1 << 6) != 0 {
                        data |= 0xFFFF_FFFF_FFFF_FF80u64.to_be();
                    }
                    i64::from_be(data as i64)
                })
            }
            n if n & 0xC0 == 0x80 => {
                (1, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(6) = lead & 0x3F;
                    std::ptr::copy_nonoverlapping(ptr, p.add(7), 1);
                    if lead & (1 << 5) != 0 {
                        data |= 0xFFFF_FFFF_FFFF_C000u64.to_be();
                    }
                    i64::from_be(data as i64)
                })
            }
            n if n & 0xE0 == 0xC0 => {
                (2, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(5) = lead & 0x1F;
                    std::ptr::copy_nonoverlapping(ptr, p.add(6), 2);
                    if lead & (1 << 4) != 0 {
                        data |= 0xFFFF_FFFF_FFE0_0000u64.to_be();
                    }
                    i64::from_be(data as i64)
                })
            }
            n if n & 0xF0 == 0xE0 => {
                (3, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(4) = lead & 0x0F;
                    std::ptr::copy_nonoverlapping(ptr, p.add(5), 3);
                    if lead & (1 << 3) != 0 {
                        data |= !0xFFFF_FFFF_F000_0000u64.to_be();
                    }
                    i64::from_be(data as i64)
                })
            }
            n if n & 0xF8 == 0xF0 => {
                (4, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(3) = lead & 0x07;
                    std::ptr::copy_nonoverlapping(ptr, p.add(4), 4);
                    if lead & (1 << 2) != 0 {
                        data |= 0xFFFF_FFF8_0000_0000u64.to_be();
                    }
                    i64::from_be(data as i64)
                })
            }
            n if n & 0xFC != 0xF8 => {
                (5, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(2) = lead & 0x03;
                    std::ptr::copy_nonoverlapping(ptr, p.add(3), 5);
                    if lead & (1 << 1) != 0 {
                        data |= 0xFFFF_FC00_0000_0000u64.to_be();
                    }
                    i64::from_be(data as i64)
                })
            }
            n if n & 0xFE != 0xFC => {
                (6, |lead, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    *p.add(1) = lead & 0x01;
                    std::ptr::copy_nonoverlapping(ptr, p.add(2), 6);
                    if lead & 1 != 0 {
                        data |= 0xFFFE_0000_0000_0000u64.to_be();
                    }
                    i64::from_be(data as i64)
                })
            }
            n if n == 0xFE => {
                (7, |_, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    std::ptr::copy_nonoverlapping(ptr, p.add(1), 7);
                    if data & 0x0080_0000_0000_0000u64.to_be() != 0 {
                        *p = 0xFF;
                    }
                    i64::from_be(data as i64)
                })
            }
            _ => {
                (8, |_, ptr| {
                    let mut data = 0u64;
                    let p = &mut data as *mut _ as *mut u8;
                    std::ptr::copy_nonoverlapping(ptr, p, 8);
                    i64::from_be(data as i64)
                })
            }
        }
    }
}

pub struct DecompressI64Iter<'a> {
    data: &'a mut i64,
    step: fn(&mut Self) -> Option<&'a mut [u8]>,
}

impl<'a> DecompressI64Iter<'a> {
    pub fn new(target: &'a mut i64) -> DecompressI64Iter<'a> {
        DecompressI64Iter {
            data: target,
            step: Self::s1,
        }
    }
    fn s1(&mut self) -> Option<&'a mut [u8]> {
        *self.data = 0;
        self.step = Self::s2;
        unsafe {
            let p = self.data as *mut _ as *mut u8;
            Some(slice::from_raw_parts_mut(p.offset(7), 1))
        }
    }
    fn s2(&mut self) -> Option<&'a mut [u8]> {
        unsafe {
            let p = self.data as *mut _ as *mut u8;
            let data: &mut u64 = std::mem::transmute(self.data as *mut _);
            match *p.offset(7) {
                n if n & 0x80 == 0 => {
                    if n & (1 << 6) != 0 {
                        *data |= 0xFFFF_FFFF_FFFF_FF80u64.to_be();
                    }
                    self.last()
                }
                n if n & 0xC0 == 0x80 => {
                    *p.offset(6) = n & 0x3F;
                    if n & (1 << 5) != 0 {
                        *data |= 0xFFFF_FFFF_FFFF_C000u64.to_be();
                    }
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(7), 1))
                }
                n if n & 0xE0 == 0xC0 => {
                    *p.offset(5) = n & 0x1F;
                    if n & (1 << 4) != 0 {
                        *data |= 0xFFFF_FFFF_FFE0_0000u64.to_be();
                    }
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(6), 2))
                }
                n if n & 0xF0 == 0xE0 => {
                    *p.offset(4) = n & 0x0F;
                    if n & (1 << 3) != 0 {
                        *data |= !0xFFFF_FFFF_F000_0000u64.to_be();
                    }
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(5), 3))
                }
                n if n & 0xF8 == 0xF0 => {
                    *p.offset(3) = n & 0x07;
                    if n & (1 << 2) != 0 {
                        *data |= 0xFFFF_FFF8_0000_0000u64.to_be();
                    }
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(4), 4))
                }
                n if n & 0xFC != 0xF8 => {
                    *p.offset(2) = n & 0x03;
                    if n & (1 << 1) != 0 {
                        *data |= 0xFFFF_FC00_0000_0000u64.to_be();
                    }
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(3), 5))
                }
                n if n & 0xFE != 0xFC => {
                    *p.offset(1) = n & 0x01;
                    if n & 1 != 0 {
                        *data |= 0xFFFE_0000_0000_0000u64.to_be();
                    }
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p.offset(2), 6))
                }
                n if n == 0xFE => {
                    self.step = Self::s3;
                    Some(slice::from_raw_parts_mut(p.offset(1), 7))
                }
                _ => {
                    self.step = Self::last;
                    Some(slice::from_raw_parts_mut(p, 8))
                }
            }
        }
    }

    fn s3(&mut self) -> Option<&'a mut [u8]> {
        unsafe {
            let data: &mut u64 = std::mem::transmute(self.data as *mut _);
            if *data & 0x0080_0000_0000_0000u64.to_be() != 0 {
                *data |= 0xFF;
            }
        }
        self.last()
    }

    fn last(&mut self) -> Option<&'a mut [u8]> {
        *self.data = i64::from_be(*self.data);
        self.step = Self::done;
        None
    }

    fn done(&mut self) -> Option<&'a mut [u8]> {
        None
    }
}

impl<'a> Iterator for DecompressI64Iter<'a> {
    type Item = &'a mut [u8];
    fn next(&mut self) -> Option<&'a mut [u8]> {
        (self.step)(self)
    }
}

impl FusedIterator for DecompressI64Iter<'_> {}


pub fn try_decompress_u64<E, F: FnMut(&mut [u8]) -> Result<(), E>>(mut f: F) -> Result<u64, E> {
    unsafe {
        let mut lead = MaybeUninit::uninit();
        f(slice::from_mut(lead.assume_init_mut()))?;
        try_decompress_u64_with_lead(lead.assume_init(), f)
    }
}

pub fn try_decompress_u64_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(
    lead: u8,
    f: F,
) -> Result<u64, E> {
    let mut data = 0u64;
    let p = &mut data as *mut _ as *mut u8;
    unsafe {
        match lead {
            n if n & 0x80 == 0 => {
                *p.offset(7) = n & 0x7F;
            }
            n if n & 0xC0 == 0x80 => {
                *p.offset(6) = n & 0x3F;
                f(slice::from_raw_parts_mut(p.offset(7), 1))?;
            }
            n if n & 0xE0 == 0xC0 => {
                *p.offset(5) = n & 0x1F;
                f(slice::from_raw_parts_mut(p.offset(6), 2))?;
            }
            n if n & 0xF0 == 0xE0 => {
                *p.offset(4) = n & 0x0F;
                f(slice::from_raw_parts_mut(p.offset(5), 3))?;
            }
            n if n & 0xF8 == 0xF0 => {
                *p.offset(3) = n & 0x07;
                f(slice::from_raw_parts_mut(p.offset(4), 4))?;
            }
            n if n & 0xFC != 0xF8 => {
                *p.offset(2) = n & 0x03;
                f(slice::from_raw_parts_mut(p.offset(3), 5))?;
            }
            n if n & 0xFE != 0xFC => {
                *p.offset(1) = n & 0x01;
                f(slice::from_raw_parts_mut(p.offset(2), 6))?;
            }
            n if n == 0xFE => {
                f(slice::from_raw_parts_mut(p.offset(1), 7))?;
            }
            _ => {
                f(slice::from_raw_parts_mut(p, 8))?;
            }
        }
    }
    Ok(u64::from_be(data))
}

pub fn try_decompress_i64<E, F: FnMut(&mut [u8]) -> Result<(), E>>(mut f: F) -> Result<i64, E> {
    unsafe {
        let mut lead = MaybeUninit::uninit();
        f(slice::from_mut(lead.assume_init_mut()))?;
        try_decompress_i64_with_lead(lead.assume_init(), f)
    }
}

pub fn try_decompress_i64_with_lead<E, F: FnOnce(&mut [u8]) -> Result<(), E>>(
    lead: u8,
    f: F,
) -> Result<i64, E> {
    let mut data = 0u64;
    let p = &mut data as *mut _ as *mut u8;
    unsafe {
        match lead {
            n if n & 0x80 == 0 => {
                *p.offset(7) = n & 0x7F;
                if n & (1 << 6) != 0 {
                    data |= 0xFFFF_FFFF_FFFF_FF80u64.to_be();
                }
            }
            n if n & 0xC0 == 0x80 => {
                *p.offset(6) = n & 0x3F;
                f(slice::from_raw_parts_mut(p.offset(7), 1))?;
                if n & (1 << 5) != 0 {
                    data |= 0xFFFF_FFFF_FFFF_C000u64.to_be();
                }
            }
            n if n & 0xE0 == 0xC0 => {
                *p.offset(5) = n & 0x1F;
                f(slice::from_raw_parts_mut(p.offset(6), 2))?;
                if n & (1 << 4) != 0 {
                    data |= 0xFFFF_FFFF_FFE0_0000u64.to_be();
                }
            }
            n if n & 0xF0 == 0xE0 => {
                *p.offset(4) = n & 0x0F;
                f(slice::from_raw_parts_mut(p.offset(5), 3))?;
                if n & (1 << 3) != 0 {
                    data |= !0xFFFF_FFFF_F000_0000u64.to_be();
                }
            }
            n if n & 0xF8 == 0xF0 => {
                *p.offset(3) = n & 0x07;
                f(slice::from_raw_parts_mut(p.offset(4), 4))?;
                if n & (1 << 2) != 0 {
                    data |= 0xFFFF_FFF8_0000_0000u64.to_be();
                }
            }
            n if n & 0xFC != 0xF8 => {
                *p.offset(2) = n & 0x03;
                f(slice::from_raw_parts_mut(p.offset(3), 5))?;
                if n & (1 << 1) != 0 {
                    data |= 0xFFFF_FC00_0000_0000u64.to_be();
                }
            }
            n if n & 0xFE != 0xFC => {
                *p.offset(1) = n & 0x01;
                f(slice::from_raw_parts_mut(p.offset(2), 6))?;
                if n & (1) != 0 {
                    data |= 0xFFFE_0000_0000_0000u64.to_be();
                }
            }
            n if n == 0xFE => {
                f(slice::from_raw_parts_mut(p.offset(1), 7))?;
                if data & 0x0800_0000_0000_0000u64.to_be() != 0 {
                    *p = 0xFF;
                }
            }
            _ => {
                f(slice::from_raw_parts_mut(p, 8))?;
            }
        }
    }
    Ok(i64::from_be(data as i64))
}