use core::{ascii::Char::{self, Null}, fmt::Display};

use crate::{arch::i686::kbd::Key::P, base::{display::{displaybuffer::DisplayError::CursorError, screencharacter::ScreenCharacter}, text::{self, sysstr, textbuffer::{self, TextBuffer}}}};

pub struct DisplayBuffer<const W: usize, const H: usize> {
    pub buffer: TextBuffer<ScreenCharacter, {W * H}>,
    pub cursor: usize,
    pub offset: usize,
}

pub enum DisplayError {
    UnknownError,
    WriteError,
    CursorError,
}

impl<const W: usize, const H: usize> DisplayBuffer<W, H> {

    pub const fn new() -> Self {
        Self {
            buffer: TextBuffer::<{W * H}>::new(ScreenCharacter::new(Char::Null, None, None, None)),
            cursor: 0,
            offset: 0,
        }
    }

    pub fn set_char(&mut self, ch: ScreenCharacter) -> Result<(), DisplayError> {
        let cur_cell = self.buffer.get_mut(self.cursor).ok_or(DisplayError::WriteError)?;
        *cur_cell = ch;
        Ok(())
    }

    pub fn write_char(&mut self, ch: ScreenCharacter) -> Result<(), DisplayError> { // set char and then advance cursor without shifting
        self.set_char(ch)?;
        if self.offset < W * H { 
            self.offset += 1;
        } else {
            return Err(DisplayError::WriteError);
        }
        self.forward_cursor()
    }

    // will probably be add at a later date
    // pub fn add_char(&mut self, ch: ScreenCharacter) -> Result<(), DisplayError> {
    //     if self.offset >= (W * H).saturating_sub(1) { return Err(DisplayError::WriteError) }
    //     else {
    //         unsafe {
    //             let src_ptr = self.buffer.get_ptr().add(self.cursor);
    //             let dst_ptr = self.buffer.get_mut_ptr().add(self.cursor + 1);
    //             core::ptr::copy(src_ptr, dst_ptr, self.offset - self.cursor);
    //         }
    //         self.cursor += 1;
    //         self.offset += 1;
    //     }
    //     Ok(())
    // }

    // pub fn del_char() {
        
    // }

    pub fn clear(&mut self) {
        unsafe {
            let buf_ptr = self.buffer.get_mut_ptr();
            core::ptr::write_bytes(buf_ptr, 0x00, BUFFER_LENGTH);
        }
        self.cursor = 0;
        self.offset = 0;
    }

    pub fn forward_cursor(&mut self) -> Result<(), DisplayError> { 
        if self.cursor >= self.offset { Err(DisplayError::CursorError) }
        else { Ok(()) }
    }

    pub fn backward_cursor(&mut self) -> Result<(), DisplayError> { 
        if self.cursor <= 0 { Err(DisplayError::CursorError) }
        else { Ok(()) }
    }

}

// access attributes
impl<const W: usize, const H: usize> DisplayBuffer<W, H> {

    // pub fn get_offset(&self) -> usize { self.offset }
    // pub fn get_cursor(&self) -> usize { self.cursor }

    pub const fn get_width(&self) -> usize { W }
    pub const fn get_height(&self) -> usize { H }

    pub fn row_of(idx: usize) -> usize { idx / W }
    pub fn col_of(idx: usize) -> usize { idx % W }

    pub fn get_cursor_row(&self) -> usize { row_of(self.cursor) }
    pub fn get_cursor_col(&self) -> usize { col_of(self.cursor) }

    pub fn get_offset_row(&self) -> usize { row_of(self.offset) }
    pub fn get_offset_col(&self) -> usize { col_of(self.offset) }

}