use crate::{arch::i686::vga::update_cursor, 
    base::{
        display::{
            displaybuffer::{self, DisplayBuffer}, displayframe::{self, DisplayFrame, FlushableBuffer, FramePointer, LastTick}, displaystr::{self, DisplayString}, inputframe::{self, InputFrame, InputToBuffer}, screencharacter::{self,ScreenCharacter}, scroll::*, strparse::*,
        }, text::{sysstr::{self, SysStr, SysStrError}, textbuffer::AsPtr}
    }
};
use crate::drivers::display::cursor::CursorFn;
use crate::arch::i686::vga;

pub const BUFFER_WIDTH: usize = 80;
pub const BUFFER_HEIGHT: usize = 200;
pub const BUFFER_CAPACITY: usize = BUFFER_WIDTH * BUFFER_HEIGHT;

const FLUSH_FRAME_WIDTH: usize = 80;
const FLUSH_FRAME_HEIGHT: usize = 25;
type FrameBuffer = [[u16; FLUSH_FRAME_WIDTH]; FLUSH_FRAME_HEIGHT];

const DISPLAY_FRAME_PTR: FramePointer<u8> = FramePointer(vga::VGA_BUFFER_ADR);

pub struct DisplayWriter {
    buffer: DisplayBuffer,
    display_frame: DisplayFrame,
    display_frame_ptr: FramePointer<FrameBuffer>,
    input_frame: InputFrame,
    cursor_fn: CursorFn,
    last_tick: LastTick,
    metadata: RowAndColMetadata,
}

impl DisplayWriter {

    pub const fn new(
        frame_ptr: *mut FrameBuffer, 
        input_frame_size: usize,
    ) -> Self {
        Self {
            buffer: DisplayBuffer::<BUFFER_WIDTH, BUFFER_HEIGHT>::new(),
            display_frame: DisplayFrame::new(FLUSH_FRAME_WIDTH, FLUSH_FRAME_HEIGHT),
            display_frame_ptr: FramePointer(frame_ptr),
            input_frame: InputFrame::new(input_frame_size),
            cursor_fn: CursorFn::define_cursor_fn(
                Some(vga::enable_cursor), 
                Some(vga::disable_cursor), 
                Some(update_cursor),
            ),
            last_tick: LastTick(0),
            metadata: RowAndColMetadata::new(),
        }
    }

    pub fn write_char(&mut self, ch: ScreenCharacter, is_auto_scroll: bool) -> Result<(), DisplayWriterError> {
        if self.buffer.offset < BUFFER_CAPACITY {
            unsafe {
                if ch.ascii_char == 0x0Au8 {
                    self.buffer.offset = (self.buffer.get_offset_row() + 1) * BUFFER_WIDTH;
                    let buf_offset = self.buffer.offset;
                    self.display_frame.idx = buf_offset;
                    self.input_frame.idx = buf_offset;
                    // add cursor updation
                    if is_auto_scroll {

                    }
                } else {
                    let char_ptr = self.buffer.as_mut_ptr();
                    core::ptr::write(char_ptr.add(self.buffer.offset), char); // change use buffer abstraction
                    self.buffer.offset += 1;
                    let buf_offset = self.buffer.offset;
                    self.buffer.cursor = buf_offset;
                    self.input_frame.idx += 1;
                    // update metadat and cursor
                    if self.buffer.get_offset_col() == 0 {
                        self.input_frame.idx = buf_offset;
                        if is_auto_scroll {
                            self.auto_scroll_down();
                            //check update metadata
                        }
                    }
                }
            }
            Ok(())
        } else {
            Err(DisplayWriterError::WriteError)
        }
    }

    pub fn write_str(&mut self, s: DisplayString) -> Result<(), DisplayWriterError> {

    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.input_frame.idx = 0;
    }

}

impl FlushableBuffer for DisplayWriter {

    fn flush(&self, dst: FramePointer<u8>) {
        unsafe {
            let src_base = self.buffer.as_ptr() as *const u16;
            let dst_base = dst.0 as *mut u16;
            let frame_row = self.metadata.flush_frame_row;
            let frame_col = self.metadata.flush_frame_col;
            for i in 0..FLUSH_FRAME_HEIGHT {
                let src_row_ptr = src_base.add((frame_row + i) * BUFFER_WIDTH + frame_col);
                let dst_row_ptr = dst_base.add(i * FLUSH_FRAME_WIDTH);
                for j in 0..FLUSH_FRAME_WIDTH {
                    let value = core::ptr::read(src_row_ptr.add(j));
                    core::ptr::write_volatile(dst_row_ptr.add(j), value);
                }
            }
        }
        self.flush_cursor();
    }

    fn get_last_tick(&mut self) -> &mut displayframe::LastTick { &mut self.last_tick }

    fn flush_cursor(&self) {
        
    }

}

impl ScrollableBuffer for DisplayWriter {

    fn scroll(&mut self, dir: ScrollDirection) {
        let frame_row = DisplayBuffer::row_of(self.display_frame.idx);
        let frame_col = DisplayBuffer::col_of(self.display_frame.idx);
        match dir {
            ScrollDirection::Up => {
                if frame_row > 0 {
                    self.display_frame.idx = DisplayBuffer::calculate_offset(frame_row - 1, frame_col);
                }
            },
            ScrollDirection::Left => {
                if frame_col > 0 {
                    self.display_frame.idx = DisplayBuffer::calculate_offset(frame_row, frame_col - 1);
                }
            },
            ScrollDirection::Down => {
                if frame_row + FLUSH_FRAME_HEIGHT < BUFFER_HEIGHT {
                    self.display_frame.idx = DisplayBuffer::calculate_offset(frame_row + 1, frame_col);
                }
            },
            ScrollDirection::Right => {
                if frame_col + FLUSH_FRAME_WIDTH < BUFFER_WIDTH {
                    self.display_frame.idx = DisplayBuffer::calculate_offset(frame_row, frame_col + 1);
                }
            }
        }
        self.update_metadata(); // check
    }

    fn snap_to_cursor(&mut self, snap_row: bool, snap_col: bool) {
        
    }

    fn try_snap_to_cursor(&mut self) {
        
    }

    fn auto_scroll_down(&mut self) {
        
    }

}

impl InputToBuffer for DisplayBuffer {

    fn write_from_input() {
        
    }

}

#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum DisplayWriterError {
    #[default] UnknownError,
    WriteError,
    InvalidASCIIError,  // remove this later
    CopyFromInputError,
    ScrollError,
    CursorError,
}

#[repr(C)]
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