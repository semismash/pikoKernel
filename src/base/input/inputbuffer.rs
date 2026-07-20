use core::ascii::Char;

use crate::base::text::textbuffer::{self, TextBuffer};

pub struct InputBuffer<const N: usize> {
    buffer: TextBuffer<Char>,
    cursor: usize,
    offset: usize,
}

pub enum InputError {
    UnknownError,
    WriteError,
}

impl<const N: usize> InputBuffer<N> {

    pub const fn new() -> Self {
        Self {
            buffer: TextBuffer::<N>::new(Char::Null),
            cursor: 0,
            offset: 0,
        }
    }

    pub fn write_char(&mut self, ch: Char) -> Result<(), InputError> {
        if self.cursor >= N.saturating_sub(1) { return Err(InputError::WriteError); }
        if self.offset < N {
            unsafe {
                let mut cursor_ptr = self.buffer.get_mut_ptr().add(self.cursor);
                core::ptr::copy(cursor_ptr, cursor_ptr.add(1), self.offset - self.cursor);
                core::ptr::write(cursor_ptr, ch);
                self.cursor += 1;
                self.offset += 1;
            }
            Ok(())
        } else {
            Err(InputError::WriteError)
        }
    }

    pub fn insert_char(&mut self, ch: Char) -> Result<(), InputError> {
        let cur_cell = self.buffer.get_mut(pos).ok_or(InputError::WriteError)?;
        *cur_cell = ch;
        self.forward_cursor()
    }

    pub fn del_char(&mut self) {
        if (self.cursor > 0) {
            unsafe {
                let src_ptr: *const Char = &self.buffer[self.cursor] as *const Char;
                let dest_ptr: *mut Char = &mut self.buffer[self.cursor - 1] as *mut Char;
                core::ptr::copy(src_ptr, dest_ptr, N - self.cursor);
                self.buffer[N - 1] = Char::Null; // set final slot to null
                self.cursor -= 1;
                self.offset -= 1;
            }
        }
    }

    pub fn back_char(&mut self) {
        if (self.cursor != self.offset && self.cursor < N - 1) { 
            unsafe {
                let cursor_ptr = &mut self.buffer[self.cursor] as *mut Char;
                core::ptr::copy(cursor_ptr.add(1), cursor_ptr, N - self.cursor - 1);
                self.offset -= 1;
            }
        }
    }

    pub fn clear_buffer (&mut self) {
        unsafe {
            let buf_ptr = self.buffer.as_mut_ptr(); 
            core::ptr::write_bytes(buf_ptr, 0x00, BUFFER_LENGTH);   //Char::Null = 0x00
        }
        self.cursor = 0;
        self.offset = 0;
    }

    pub fn new_line(&mut self) -> Result<(), InputError> {
        self.write_char(Char::LineFeed)
    }

    pub fn is_full(&self) -> bool { (BUFFER_LENGTH - self.offset) - 1 <= 0 }

}

// for cursor
impl<const N: usize> InputBuffer<N> {

    pub fn forward_cursor(&mut self) -> Result<(), InputError> { 
        if self.cursor >= self.offset { Err(InputError::CursorError) }
        else { Ok(()) }
    }

    pub fn backward_cursor(&mut self) -> Result<(), InputError> { 
        if self.cursor <= 0 { Err(InputError::CursorError) }
        else { Ok(()) }
    }

    pub fn move_cursor(&mut self, dir: MoveDirection) {
        match dir {
            MoveDirection::Left => {
                if self.cursor > 0 { self.cursor -= 1; }
            },
            MoveDirection::Right => {
                if self.cursor < self.offset { self.cursor += 1; }
            },
            MoveDirection::Up => {
                if self.cursor == 0 { return; }
                let (cur_row, cur_col) = self.visual_pos_of(self.cursor);
                if cur_row == 0 { self.cursor = 0; return; }
                self.cursor = self.find_cursor_at_visual(cur_row - 1, cur_col);
            },
            MoveDirection::Down => {
                if self.cursor >= self.offset { return; }
                let (cur_row, cur_col) = self.visual_pos_of(self.cursor);
                self.cursor = self.find_cursor_at_visual(cur_row + 1, cur_col);
            },
        }
    }

}

// helpers
impl<const N: usize> InputBuffer<N> {

    // convert to visual position first, to get a better idea of where the cursor will be
    fn visual_pos_of(&self, buf_cursor: usize) -> (usize, usize) {
        let mut row = 0;
        let mut col = 0;
        for i in 0..buf_cursor.min(self.offset) {
            if *self.buffer.get(i).unwrap() == Char::LineFeed {
                row += 1; col = 0;
            } else {
                col += 1;
                if col >= crate::drivers::display_old::BUFFER_WIDTH { col = 0; row += 1; }
            }
        }
        (row, col)
    }

    // find the corresponding cursor value using the info we have, to set the new current cursor position
    fn find_cursor_at_visual(&self, target_row: usize, target_col: usize) -> usize {
        let mut row = 0;
        let mut col = 0;
        let mut last_on_row: Option<usize> = None;
        for i in 0..=self.offset {
            if row == target_row {
                if col == target_col { return i; }
                last_on_row = Some(i);  // keep updating
            } else if row > target_row {
                return last_on_row.unwrap_or(self.offset);  // clamp to end of left row
            }
            if i == self.offset { break; }
            if *self.buffer.get(i).unwrap() == Char::LineFeed {
                row += 1; col = 0;
            } else {
                col += 1;
                if col >= crate::drivers::display_old::BUFFER_WIDTH { col = 0; row += 1; }
            }
        }
        last_on_row.unwrap_or(self.offset)  // target row past content, or clamp to row end
    }

}