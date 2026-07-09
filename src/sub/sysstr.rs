use core::ascii::Char;

const STR_LENGTH_DEFAULT: usize = 128;
const MAX_CAPACITY: usize = 1024;

pub struct SysStr<const N: usize = STR_LENGTH_DEFAULT> {
    container: [Char; N],
    len: usize, //pointer to the last element, can point to position out of buffer so BE CAREFUL
}

pub enum SysStrError {
    UnknownError,
    PushError,
    InvalidLenError,
}

impl<const N: usize> SysStr<N> {

    const fn new(size: usize) -> Option<Self> {
        if N > MAX_CAPACITY { return None; }
        Some(Self {
            container: [Char::Null; N],
            len: 0
        })
    }

    const fn from_str(str_in: &str) -> Option<Self>  {
        if str_in.is_empty() || !str_in.is_ascii() {
            return None
        }

        let mut buf = [Char::Null; MAX_CAPACITY];
        let mut char_iter = str_in.chars();

        let len = str_in.len();
        for ch in buf.iter_mut().take(len) {
            let raw_ch = char_iter.next().unwrap() as u32 as u8;
            *ch = Char::try_from(raw_ch).unwrap();
        } 

        Some(Self {
            container: buf,
            len: len,
        })
    }
    
}

// access
impl<const N: usize> SysStr<N> {

    fn as_bytes(&self) -> &[u8] {
        let src_ptr = self.container.as_ptr();
        let bytes = unsafe { core::slice::from_raw_parts(stc_ptr as *const u8 , self.len) };
        bytes
    }

    fn as_str(&self) -> &str {
        let bytes = self.as_bytes();
        let str_slice = core::str::from_utf8(bytes).unwrap();   //char is always 8-bit, guaranteed to not
        str_slice
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn capacity(&self) -> usize {
        N
    }

}

// mutation
impl<const N: usize> SysStr<N> {

    fn push(&mut self, ch: Char) -> Result<(), SysStrError> {
        if len >= N { return Err(SysStrError::PushError) }
        self.container[len] = ch;
        Ok(())
    }

    fn append(&mut self, str_in: &SysStr) -> Result<(), SysStrError> {
        let src_len = str_in.len();
        if src_len > N - self.len { return Err(SysStrError::PushError); }
        let src_ptr = str_in.container.as_ptr();
        let dst_ptr = self.container.as_mut_ptr();
        unsafe { core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, src_len); }
        self.len += src_len;
        Ok(())
    }

    fn push_str(&mut self, str_in: &str) -> Result<(), SysStrError> {
        let new_str = Self::from_str(str_in)?;
        let _ = self.append(&new_str)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn pop(&mut self) -> Option<Char> {
        if len == 0 { return None; }
        len -= 1;
        let ch = self.container[len as usize];  // as usize to get rust analyzer to work and detect the type
        self.container[len] = Char::Null;
        Some(ch)
    }

    fn truncate(&mut self, new_len: usize) -> Result<(), SysStrError> { // WARNING: may result in data loss
        if new_len == 0 || new_len > self.len { return Err(SysStrError::InvalidLenError); }
        self.len = new_len;
        Ok(())
    }

}