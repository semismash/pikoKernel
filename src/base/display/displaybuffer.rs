use core::{ascii::Char::{self, Null}, fmt::Display};

use crate::{arch::i686::kbd::Key::P, base::{display::{displaybuffer::DisplayError::CursorError, screencharacter::ScreenCharacter}, text::{self, sysstr, textbuffer::{self, TextBuffer}}}};

pub struct DisplayBuffer<const W: usize, const H: usize> {
    buffer: TextBuffer<ScreenCharacter, {W * H}>,
    cursor: usize,
    offset: usize,
    metadata: RowAndColMetadata, // reconsider this addition
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
            metadata: RowAndColMetadata::new(),
        }
    }

    pub fn set_char(&mut self, ch: ScreenCharacter) -> Result<(), DisplayError> {
        let cur_cell = self.buffer.get_mut(self.cursor).ok_or(DisplayError::WriteError)?;
        *cur_cell = ch;
        Ok(())
    }

    pub fn write_char(&mut self, ch: ScreenCharacter) -> Result<(), DisplayError> { // set char and then advance cursor without shifting
        self.set_char(ch)?;
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

    fn update_metadata() {

    }

}

// access attributes
impl<const W: usize, const H: usize> DisplayBuffer<W, H> {

    pub const fn get_width(&self) -> usize { W }
    pub const fn get_height(&self) -> usize { H }

    pub fn row_of(idx: usize) -> usize { idx / W }
    pub fn col_of(idx: usize) -> usize { idx % W }

    pub fn get_cursor_row(&self) -> usize { row_of(self.cursor) }
    pub fn get_cursor_col(&self) -> usize { col_of(self.cursor) }

    pub fn get_offset_row(&self) -> usize { row_of(self.offset) }
    pub fn get_offset_col(&self) -> usize { col_of(self.offset) }

}

//reconsider implementation
struct RowAndColMetadata {
    cursor_row: usize,
    cursor_col: usize,
    flush_frame_row: usize,
    flush_frame_col: usize,
}

impl RowAndColMetadata {

    const fn new() -> Self {
        Self {
            cursor_row: 0,
            cursor_col: 0,
            flush_frame_row: 0,
            flush_frame_col: 0,
        }
    }

    fn from_offset(cursor_offset: usize, flush_frame_offset: usize) -> Self {
        Self {
            cursor_row: cursor_offset / BUFFER_WIDTH,
            cursor_col: cursor_offset % BUFFER_WIDTH,
            flush_frame_row: flush_frame_offset / BUFFER_WIDTH,
            flush_frame_col: flush_frame_offset % BUFFER_WIDTH,
        }
    }

    fn from_cur_values(display_writer: &DisplayWriter) -> Self {
        Self::from_offset(display_writer.cursor_idx, display_writer.flush_frame_ptr)
    }

}