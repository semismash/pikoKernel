use core::{ascii::Char, fmt::Display};

use crate::base::{display::screencharacter::ScreenCharacter, text::{self, sysstr, textbuffer::{self, TextBuffer}}};

pub struct DisplayBuffer<const W: usize, const H: usize> {
    buffer: TextBuffer<Char, {W * H}>,
    cursor: usize,
    offset: usize,
    metadata: RowAndColMetadata::new(), // reconsider this addition
}

pub enum DisplayError {
    UnknownError,
    WriteError,
}

impl DisplayBuffer {

    pub const fn new() -> Self {
        Self {
            buffer: TextBuffer::new(fill),
            cursor: 0,
            offset: 0,
            metadata: 0,
        }
    }

    pub fn add_char(&mut self, ch: ScreenCharacter) {

    }

    pub fn del_char() {

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