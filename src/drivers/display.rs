use core::{ascii::Char};
use core::ptr::{write, write_volatile};
use core::fmt;
use crate::sys;
//use core::cell::SyncUnsafeCell;

pub const BUFFER_WIDTH: usize = 80;
pub const BUFFER_HEIGHT: usize = 25;
const BUFFER_CAPACITY: usize = BUFFER_WIDTH * BUFFER_HEIGHT;

type Buffer = [[ScreenCharacter; BUFFER_WIDTH]; BUFFER_HEIGHT];
type FrameBuffer = [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT];

#[derive(Debug, Clone, Copy)]
pub struct FramePointer(pub *mut FrameBuffer);
unsafe impl Sync for FramePointer {}

#[repr(C)]
pub struct DisplayWriter {
    buffer: Buffer,
    row_pos: usize,
    col_pos: usize,
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
pub enum VGAError {
    #[default] UnknownError,
    WriteError,
    InvalidASCIIError,
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

    pub const fn new(on_cursor_update: Option<fn(usize, usize)>) -> Self {
        Self {
            buffer: [[ScreenCharacter { ascii_char: 0x20, attribute: 0x0F, }; BUFFER_WIDTH]; BUFFER_HEIGHT],
            row_pos: 0,
            col_pos: 0,
            on_cursor_update: on_cursor_update,
            last_tick: 0,
        }
    }

    pub fn write_char_to_buf(&mut self, char: ScreenCharacter) -> Result<(), VGAError> {
        if self.get_offset() < BUFFER_CAPACITY {
            unsafe {
                if char.ascii_char == 0x0A { /* \n hex is 0x0A */
                    self.row_pos += 1;
                    self.col_pos = 0;
                } else {
                    if self.col_pos >= BUFFER_WIDTH {
                        self.row_pos += 1;
                        self.col_pos = 0;
                    }
                    let char_ptr = self.buffer
                        .get_unchecked_mut(self.row_pos)
                        .get_unchecked_mut(self.col_pos)
                        as *mut ScreenCharacter;
                    core::ptr::write(char_ptr, char);
                    self.col_pos += 1;
                }
                Ok(())
            }
        } else {
            Err(VGAError::WriteError)
        }
    }

    pub fn write_fmt_text_to_buf<FG, BG, BL>
    (
        &mut self, 
        text: &str, 
        fg_color: FG, 
        bg_color: BG, 
        blink: BL) 
    -> Result<(), VGAError>
    where
        FG: Into<Option<ForegroundColor>> + Copy,
        BG: Into<Option<BackgroundColor>> + Copy,
        BL: Into<Option<bool>> + Copy,
    {
        if !text.is_ascii() {
            Err(VGAError::InvalidASCIIError)
        } else {
            if text.len() <= (BUFFER_CAPACITY) - self.get_offset() {
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
                Err(VGAError::WriteError)
            }
        }
    }

    unsafe fn flush(&self, frame_buf: FramePointer) {
        unsafe {
            let src_ptr = self.buffer.as_ptr() as *const u16;
            let dst_ptr = frame_buf.0.as_mut_ptr() as *mut u16;
            for i in 0..(BUFFER_CAPACITY) {
                let value = core::ptr::read(src_ptr.add(i));
                core::ptr::write_volatile(dst_ptr.add(i), value);
            }
        }
        if let Some(fn_cursor_update) = self.on_cursor_update {
            let checked_row = if self.row_pos >= BUFFER_HEIGHT { BUFFER_HEIGHT - 1 } else { self.row_pos };
            fn_cursor_update(checked_row, self.col_pos);
        }
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
            for i in 0..BUFFER_HEIGHT {
                for j in 0..BUFFER_WIDTH {
                    let buf_ptr = self.buffer
                        .get_unchecked_mut(i)
                        .get_unchecked_mut(j) as *mut ScreenCharacter;
                    core::ptr::write(
                        buf_ptr, 
                        ScreenCharacter { 
                            ascii_char: 0x20,
                            attribute: 0x0F,
                        }
                    );
                }
            }
        }
        self.row_pos = 0;
        self.col_pos = 0;
    }

    fn get_offset(&self) -> usize {
        (self.row_pos * BUFFER_WIDTH) + self.col_pos
    }

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
        if (BUFFER_CAPACITY - self.get_offset()) <= 0 {
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