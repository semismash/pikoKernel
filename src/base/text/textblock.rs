use crate::base::text::textbuffer::{self, TextBuffer};

pub struct TextBlock<T, const W: usize, const H: usize> {
    buffer: TextBuffer<T, {W * H}>,
    cursor: usize,  // position to next buffer cell write
}

pub enum TextBlockError {
    UnknownError,
    BufferFull,
}

impl<T, const W: usize, const H: usize> TextBlock<T, W, H> {

    pub const fn new() -> Self {
        Self {
            buffer: TextBuffer::new(ScreenCharacter::blank()),
            cursor: 0,
        }
    }

}

// access attributes
impl<T, const W: usize, const H: usize> TextBlock<T, W, H> {

    pub const fn get_width(&self) -> usize { W }
    pub const fn get_height(&self) -> usize { H }

    pub fn row_of(idx: usize) -> usize { idx / W }
    pub fn col_of(idx: usize) -> usize { idx % W }

}

// access and mutation
impl<T, const W: usize, const H: usize> TextBlock<T, W, H> {

    pub fn write_cell(&mut self, item: T) -> Result<(), TextBlockError> {
        self.buffer.set(self.cursor, item).map_err(|_| TextBlockError::BufferFull)?;
        self.cursor += 1;
        Ok(())
    }

    pub fn insert_cell(&mut self, item: T) -> Result<(), TextBlockError> {
        if self.cursor >= W * H { return Err(TextBlockError::BufferFull); }
        unsafe { core::ptr::copy_nonoverlapping(src, dst, count); }
        self.write_cell(T)
    }

}

