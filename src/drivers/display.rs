use core::ops::Add;
use core::{ascii::Char};
use core::ptr::{write, write_volatile};
use core::fmt;
use crate::arch::i686::vga::update_cursor;
use crate::drivers::ScrollDirection::Up;
use crate::sys;
use crate::drivers::input;
use crate::drivers::input::InputBuffer;
//use core::cell::SyncUnsafeCell;

pub const BUFFER_WIDTH: usize = 80;
pub const BUFFER_HEIGHT: usize = 50;
pub const BUFFER_CAPACITY: usize = BUFFER_WIDTH * BUFFER_HEIGHT;

// flush frame dimensions
pub const FLUSH_FRAME_WIDTH: usize = 80;
pub const FLUSH_FRAME_HEIGHT: usize = 25;

pub const CURSOR_START: u8 = 14;
pub const CURSOR_END: u8 = 15;

// MAKE SURE FLUSH FRAME DIMENSIONS ARE NOT LARGER THAN THE BUFFER ITSELF
const _: u8 = [0][(FLUSH_FRAME_WIDTH > BUFFER_WIDTH) as usize];
const _: u8 = [0][(FLUSH_FRAME_HEIGHT > BUFFER_HEIGHT) as usize];

type Buffer = [[ScreenCharacter; BUFFER_WIDTH]; BUFFER_HEIGHT];
type FrameBuffer = [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT];

#[derive(Debug, Clone, Copy)]
pub struct FramePointer(pub *mut FrameBuffer);
unsafe impl Sync for FramePointer {}

#[repr(C)]
pub struct DisplayWriter {
    buffer: Buffer,
    offset: usize,
    flush_frame_ptr: usize, // top right corner of the flush frame
    input_frame_ptr: usize,
    cursor_idx: usize,
    on_cursor_enable: Option<fn(u8, u8)>,
    on_cursor_disable: Option<fn()>,
    on_cursor_update: Option<fn(usize, usize)>,
    last_tick: u32, // for concurrency and synchronization
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(2))]
pub struct ScreenCharacter {
    ascii_char: u8,
    attribute: u8,
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum DisplayError {
    #[default] UnknownError,
    WriteError,
    InvalidASCIIError,
    CopyFromInputError,
    ScrollError,
    CursorError,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ForegroundColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    #[default] White = 15,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum BackgroundColor {
    #[default] Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,  
    Gray = 7,
}

pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScreenCharacter {
    
    pub fn new<FG, BG, BL>
    (
        ascii_code: Char,
        fg_color: FG,
        bg_color: BG,
        blink: BL,
    ) 
    -> Self
    where
        FG: Into<Option<ForegroundColor>>,
        BG: Into<Option<BackgroundColor>>,
        BL: Into<Option<bool>>,
    {
        Self {
            ascii_char: ascii_code.to_u8(),
            attribute: (blink.into().unwrap_or(false) as u8) << 7 | 
                (bg_color.into().unwrap_or(BackgroundColor::default()) as u8) << 4 | 
                (fg_color.into().unwrap_or(ForegroundColor::default()) as u8),
        }
    }

}

impl DisplayWriter {

    pub const fn new(
            on_cursor_enable: Option<fn(u8, u8)>, 
            on_cursor_disable: Option<fn()>,
            on_cursor_update: Option<fn(usize, usize)>
        ) -> Self {
        Self {
            buffer: [[ScreenCharacter { ascii_char: 0x20, attribute: 0x0F, }; BUFFER_WIDTH]; BUFFER_HEIGHT],
            offset: 0,
            flush_frame_ptr: 0,
            input_frame_ptr: 0,
            cursor_idx: 0,
            on_cursor_enable: on_cursor_enable,
            on_cursor_disable: on_cursor_disable,
            on_cursor_update: on_cursor_update,
            last_tick: 0,
        }
    }

    pub fn write_char_to_buf(&mut self, char: ScreenCharacter) -> Result<(), DisplayError> {
        if self.offset < BUFFER_CAPACITY {
            unsafe {
                if char.ascii_char == 0x0A { /* \n hex is 0x0A */
                    self.offset = (Self::get_row(self.offset) + 1) * BUFFER_WIDTH;
                    //self.input_frame_ptr = self.offset;
                } else {
                    let col_pos = Self::get_col(self.offset);
                    let char_ptr = self.buffer.as_mut_ptr() as *mut ScreenCharacter;
                    core::ptr::write(char_ptr.add(self.offset), char);
                    self.offset += 1;
                    //self.input_frame_ptr += 1;
                    /*if Self::get_col(self.offset) == 0 {
                        self.input_frame_ptr = self.offset;
                    }*/
                }
                self.input_frame_ptr = self.offset; // TO BE RECHECKED
                self.cursor_idx = self.offset;  // CHANGE
                Ok(())
            }
        } else {
            Err(DisplayError::WriteError)
        }
    }

    pub fn write_fmt_text_to_buf<FG, BG, BL>
    (
        &mut self, 
        text: &str, 
        fg_color: FG, 
        bg_color: BG, 
        blink: BL) 
    -> Result<(), DisplayError>
    where
        FG: Into<Option<ForegroundColor>> + Copy,
        BG: Into<Option<BackgroundColor>> + Copy,
        BL: Into<Option<bool>> + Copy,
    {
        if !text.is_ascii() {
            Err(DisplayError::InvalidASCIIError)
        } else {
            if text.len() <= (BUFFER_CAPACITY) - self.offset {
                for ch in text.chars() {
                    let ascii_ch = unsafe { core::ascii::Char::from_u8_unchecked(ch as u8) };
                    let screen_ch = ScreenCharacter::new(
                        ascii_ch, 
                        fg_color.into().unwrap_or(ForegroundColor::default()), 
                        bg_color.into().unwrap_or(BackgroundColor::default()),
                        blink.into().unwrap_or(false),
                    );
                    self.write_char_to_buf(screen_ch)?;
                }
                Ok(())
            } else {
                Err(DisplayError::WriteError)
            }
        }
    }

    unsafe fn flush(&self, frame_buf: FramePointer) -> Result<(), DisplayError> {
        unsafe {
            let src_ptr = self.buffer[self.flush_frame_ptr].as_ptr() as *const u16;
            let dst_ptr = frame_buf.0.as_mut_ptr() as *mut u16;
            for i in 0..(BUFFER_CAPACITY) {
                let value = core::ptr::read(src_ptr.add(i));
                core::ptr::write_volatile(dst_ptr.add(i), value);
            }
            self.flush_cursor()
        }
        // cursor updation code
        
    }

    fn flush_cursor(&self) -> Result<(), DisplayError> {
        let fn_cursor_enable = self.on_cursor_enable.ok_or(DisplayError::CursorError)?;
        let fn_cursor_disable = self.on_cursor_disable.ok_or(DisplayError::CursorError)?;
        let fn_cursor_update = self.on_cursor_update.ok_or(DisplayError::CursorError)?;
        // let row = self.cursor_idx / BUFFER_WIDTH;   // RECHECK
        // let checked_row = if row >= BUFFER_HEIGHT { BUFFER_HEIGHT - 1 } else { row };
        // let col = self.cursor_idx % BUFFER_WIDTH;   // RECHECK
        // fn_cursor_update(checked_row, col);
        let cursor_row = self.cursor_idx / BUFFER_WIDTH;
        let cursor_col = self.cursor_idx % BUFFER_WIDTH;
        let frame_row = self.flush_frame_ptr / BUFFER_WIDTH;
        let frame_col = self.flush_frame_ptr % BUFFER_WIDTH;
        let display_cursor_row = {
            let display_diff = cursor_row - frame_row;
            if cursor_row < frame_row || cursor_row - frame_row > BUFFER_HEIGHT { Some(display_diff) }
            else { None }
        };
        let display_cursor_col = {
            let display_diff = cursor_col - frame_col;
            if cursor_col < frame_col || cursor_col - frame_col > BUFFER_HEIGHT { Some(display_diff) }
            else { None }
        };
        if display_cursor_row.is_some() && display_cursor_col.is_some() {
            fn_cursor_enable(CURSOR_START, CURSOR_END);
            fn_cursor_update(display_cursor_row.unwrap(), display_cursor_col.unwrap());
        } else {
            fn_cursor_disable;
        }
        Ok(())
    }

    pub unsafe fn flush_sync(&mut self, frame_buf: FramePointer) {
        let mut last = self.last_tick;
        crate::sys::time::SysTime::on_tick(&mut last, || {
            self.flush(frame_buf);
        });
        self.last_tick = last;
    }

    pub fn clear(&mut self) {
        unsafe {
            let buf_ptr = self.buffer.as_mut_ptr() as *mut ScreenCharacter;
            for i in 0..BUFFER_CAPACITY {
                core::ptr::write(
                buf_ptr.add(i),
                ScreenCharacter { 
                        ascii_char: 0x20,
                        attribute: 0x0F,
                    }
                )
            }
        }
        self.offset = 0;
        self.cursor_idx = 0;
        self.input_frame_ptr = self.offset;
    }

    /*pub fn try_scroll(&mut self, dir: ScrollDirection) -> Result<(), DisplayError> {
        
    }*/

    pub fn scroll(&mut self, dir: ScrollDirection) {
        let frame_row = Self::get_row(self.flush_frame_ptr);
        let frame_col = Self::get_col(self.flush_frame_ptr);
        match dir {
            ScrollDirection::Up => {
                if frame_row > 0 {
                    let new_frame = Self::calculate_offset(frame_row - 1, frame_col);
                    self.flush_frame_ptr = new_frame;
                    unsafe { self.update_flush_frame(new_frame); }
                }
            },
            ScrollDirection::Left => {
                if frame_col > 0 {
                    let new_frame = Self::calculate_offset(frame_row, frame_col - 1);
                    self.flush_frame_ptr = new_frame;
                    unsafe { self.update_flush_frame(new_frame); }
                }
            },
            ScrollDirection::Down => {
                if frame_row < BUFFER_HEIGHT - frame_row {
                    let new_frame = Self::calculate_offset(frame_row + 1, frame_col);
                    self.flush_frame_ptr = new_frame;
                    unsafe { self.update_flush_frame(new_frame); }
                }
            },
            ScrollDirection::Right => {
                if frame_col < BUFFER_WIDTH - frame_col {
                    let new_frame = Self::calculate_offset(frame_row, frame_col + 1);
                    self.flush_frame_ptr = new_frame;
                    unsafe { self.update_flush_frame(new_frame); }
                }
            }
        }
    }

    unsafe fn update_flush_frame(&mut self, new_offset: usize) {
        self.flush_frame_ptr = new_offset;
    }

    //helper
    pub fn get_offset(&self) -> usize { self.offset }
    fn calculate_offset(row: usize, col: usize) -> usize { row * BUFFER_WIDTH + col }
    pub fn get_row(idx: usize) -> usize { idx / BUFFER_WIDTH }
    pub fn get_col(idx: usize) -> usize { idx % BUFFER_WIDTH }
    pub fn get_input_frame_ptr(&self) -> usize { self.input_frame_ptr }

}

macro_rules! to_buf {
    ($buf:expr, $txt:expr) => {
        $crate::drivers::display::to_buf!($buf, $txt, None, None, None)
    };
    ($buf:expr, $txt:expr, $fg:expr) => {
        $crate::drivers::display::to_buf!($buf, $txt, $fg, None, None)
    };
    ($buf:expr, $txt:expr, $fg:expr, $bg:expr) => {
        $crate::drivers::display::to_buf!($buf, $txt, $fg, $bg, None)
    };
    ($buf:expr, $txt:expr, $fg:expr, $bg:expr, $bl:expr) => {
        {
            $buf.write_fmt_text_to_buf($txt, $fg, $bg, $bl)
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed!");
    };
}
pub(crate) use to_buf;

macro_rules! print {
    ($buf:expr, $frame:expr, $($args:tt)*) => {
        {
            let res = $crate::drivers::display::to_buf!($buf, $($args)*);
            if res.is_ok() {
                unsafe { $buf.flush_sync($frame); }
            }
            res
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::drivers::display::print!");
    };
}
pub(crate) use print;

macro_rules! println {
    ($buf:expr, $frame:expr, $($args:tt)*) => {
        {
            let res = $crate::drivers::display::to_buf!($buf, $($args)*);
            if res.is_ok() {
                let nl_char = 
                $crate::drivers::display::ScreenCharacter::new(
                    core::ascii::Char::LineFeed,
                    None,
                    None,
                    None,
                );
                let nl_res = $buf.write_char_to_buf(nl_char);
                if nl_res.is_ok() {
                    unsafe { $buf.flush_sync($frame); }
                }
                nl_res
            } else {
                res
            }
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::drivers::display::println!");
    };
}
pub(crate) use println;

impl DisplayWriter {

    pub fn clear_screen(&mut self, frame_buf: FramePointer) {
        unsafe {
            self.clear();
            self.flush_sync(frame_buf);
        }
    }

    pub fn check_if_full(&self) -> bool {
        if (BUFFER_CAPACITY - self.offset) - 1 <= 0 {
            true
        } else {
            false
        }
    }

}

impl fmt::Write for DisplayWriter {

    fn write_str(&mut self, s: &str) -> fmt::Result { // debugging purpose only for now
        self.write_fmt_text_to_buf(s, None, None, None)
            .map_err(|_| fmt::Error)
    }

}

macro_rules! write_and_flush {
    ($buf:expr, $frame:expr) => { 
        unsafe {
            $buf.flush_sync($frame);
        }
    };
    ($buf:expr, $frame:expr, $fmt:expr $(, $($args:tt)*)?) => {
        {
            use core::fmt::Write;
            core::write!($buf, $fmt $(, $($args)*)?)
                .map(|_| $buf.flush_sync($frame))
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::drivers::display::write_and_flush!");
    };
}
pub(crate) use write_and_flush;

impl DisplayWriter {

    pub fn write_from_input_buf(&mut self, input_buf: &InputBuffer) -> Result<(), DisplayError> {
        let input_offset = input_buf.offset;
        let frame_idx = self.input_frame_ptr;
        let MAX_SAFE_CAPACITY = BUFFER_CAPACITY - 1;
        if frame_idx >= MAX_SAFE_CAPACITY { return Err(DisplayError::CopyFromInputError); }
        let remaining_capacity = MAX_SAFE_CAPACITY - frame_idx;
        if input_offset < remaining_capacity {
            let mut flush_amt = {
                if input::BUFFER_LENGTH < remaining_capacity { input::BUFFER_LENGTH }
                else { remaining_capacity }
            };
            unsafe {
                let base_ptr: *mut ScreenCharacter = self.buffer.as_mut_ptr() as *mut ScreenCharacter;
                let frame_ptr: *mut ScreenCharacter = base_ptr.add(frame_idx);
                let input_ptr: *const Char = input_buf.buffer.as_ptr();
                let mut i = 0;  //input
                let mut j = 0;  //display
                let mut cur_col = self.input_frame_ptr % BUFFER_WIDTH;
                let mut cursor_j: Option<usize> = None;   // none means not yet found
                let mut real_end_j: Option<usize> = None;

                while i < flush_amt {
                    if i == input_buf.idx { cursor_j = Some(j); }
                    if i == input_buf.offset { real_end_j = Some(j); }

                    let fit = remaining_capacity - j;
                    if flush_amt - i > fit { flush_amt = fit + i; }

                    let cur_ch = *input_ptr.add(i);
                    if cur_ch == Char::LineFeed {
                        let remaining_slots_in_row = BUFFER_WIDTH - cur_col;
                        if j + remaining_slots_in_row >= remaining_capacity {
                            break;    // halt immediately to prevent buffer overflow
                        }
                        for k in 0..remaining_slots_in_row {
                            core::ptr::write(
                                base_ptr.add(frame_idx + j + k),
                                ScreenCharacter { ascii_char: 0x20, attribute: 0x0F },
                            );
                        }
                        j += remaining_slots_in_row;
                        cur_col = 0;
                    } else {
                        core::ptr::write(
                            base_ptr.add(frame_idx + j),
                            ScreenCharacter { 
                                ascii_char: (*input_ptr.add(i)).to_u8(), 
                                attribute: 0x0F, 
                            }
                        );
                        j += 1;
                        cur_col += 1;
                        if cur_col >= 80 { cur_col = 0; }
                    }
                    i += 1;
                }
                // if idx == offset, it means the cursor check never fired inside the loop, so default to end
                let real_end_j = real_end_j.unwrap_or(j);
                self.cursor_idx = frame_idx + cursor_j.unwrap_or(real_end_j);   // CHANGE
                if i < input_buf.offset {
                    self.offset = MAX_SAFE_CAPACITY;
                } else {
                    self.offset = frame_idx + real_end_j;
                }
            }
            Ok(())
        } else {
            Err(DisplayError::CopyFromInputError)
        }
    }

}