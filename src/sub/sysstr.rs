use core::ascii::Char;
use core::fmt::{self, Display, Write};
use core::iter::{Copied, Map};
use core::ops::Deref;
use core::slice::Iter;

use crate::sub::sysstr::SysStrError::InvalidASCII;

const STR_LENGTH_DEFAULT: usize = 128;
const MAX_CAPACITY: usize = 1024;

#[derive(Debug)]
pub struct SysStr<const N: usize = STR_LENGTH_DEFAULT> {
    container: [Char; N],
    len: usize, //pointer to the last element, can point to position out of buffer so BE CAREFUL
}

#[derive(Debug, Clone, Copy)]
pub enum SysStrError {
    UnknownError,
    CapacityExceeded,
    InvalidLenError,
    InvalidASCII,
}

impl<const N: usize> SysStr<N> {

    pub const fn new() -> Self {
        assert!(N <= MAX_CAPACITY);
        Self {
            container: [Char::Null; N],
            len: 0
        }
    }

    pub fn from_str(str_in: &str) -> Option<Self> {
        if !str_in.is_ascii() || str_in.len() > N {
            return None;
        }

        let mut buf = [Char::Null; N];
        for (ch, byte) in buf.iter_mut().zip(str_in.bytes()) {
            *ch = Char::from_u8(byte)?;
        }

        Some(Self {
            container: buf,
            len: str_in.len(),
        })
    }
    
}

// access
impl<const N: usize> SysStr<N> {

    pub fn as_bytes(&self) -> &[u8] {
        let src_ptr = self.container.as_ptr();
        let bytes = unsafe { core::slice::from_raw_parts(src_ptr as *const u8 , self.len) };
        bytes
    }

    pub fn as_str(&self) -> &str {
        let bytes = self.as_bytes();
        let str_slice = unsafe{ core::str::from_utf8(bytes).unwrap_unchecked() };   //char is always 8-bit, guaranteed to not fail
        str_slice
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn chars(&self) -> Copied<Iter<'_, Char>> {
        self.container.iter().copied()
    }

}

// mutation
impl<const N: usize> SysStr<N> {

    pub fn push(&mut self, ch: Char) -> Result<(), SysStrError> {
        if self.len >= N { return Err(SysStrError::CapacityExceeded) }
        self.container[self.len] = ch;
        self.len += 1;
        Ok(())
    }

    pub fn append<const M: usize>(&mut self, str_in: &SysStr<M>) -> Result<(), SysStrError> {
        let src_len = str_in.len();
        if src_len > N - self.len { return Err(SysStrError::CapacityExceeded); }
        unsafe {
            let src_ptr = str_in.container.as_ptr();
            let dst_ptr = self.container.as_mut_ptr().add(self.len);
            core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, src_len);
        }
        self.len += src_len;
        Ok(())
    }

    pub fn push_str(&mut self, str_in: &str) -> Result<(), SysStrError> {
        let new_str = Self::from_str(str_in).ok_or(SysStrError::InvalidASCII)?;
        let _ = self.append(&new_str)?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn pop(&mut self) -> Option<Char> {
        if self.len == 0 { return None; }
        self.len -= 1;
        let ch = self.container[self.len];
        Some(ch)
    }

    pub fn truncate(&mut self, new_len: usize) -> Result<(), SysStrError> { // WARNING: may result in data loss
        if new_len == 0 || new_len > self.len { return Err(SysStrError::InvalidLenError); }
        self.len = new_len;
        Ok(())
    }

}

// formatting
impl<const N: usize> Write for SysStr<N> {

    fn write_str(&mut self, s: &str) -> fmt::Result {
        let s_len = s.len();
        if !s.is_ascii() || s_len > N - self.len {
            return Err(fmt::Error);
        }
        let dst_ptr = self.container.as_mut_ptr();
        unsafe {
            for (i, byte) in s.bytes().enumerate() {    // infallible
                core::ptr::write(dst_ptr.add(self.len + i), Char::from_u8(byte).unwrap_unchecked());
            }
        }
        self.len += s_len;
        Ok(())
    }

}

impl<const N: usize> Display for SysStr<N> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }

}

impl<const N: usize> Deref for SysStr<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }

}

impl<const N: usize, const M: usize> PartialEq<SysStr<M>> for SysStr<N> {

    fn eq(&self, other: &SysStr<M>) -> bool {
        self.as_bytes() == other.as_bytes()
    }

    fn ne(&self, other: &SysStr<M>) -> bool {
        !self.eq(other)
    }

}

impl<const N: usize> PartialEq<&str> for SysStr<N> {

    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }

    fn ne(&self, other: &&str) -> bool {
        !self.eq(other)
    }

}

impl<const N: usize> TryFrom<&str> for SysStr<N> {
    type Error = SysStrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value).ok_or(SysStrError::InvalidASCII)
    }

}