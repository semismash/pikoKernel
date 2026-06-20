use core::{ascii::Char, mem::MaybeUninit};
use core::ptr::{write, write_volatile};

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_CAPACITY: usize = BUFFER_WIDTH * BUFFER_HEIGHT;

type Buffer = [[MaybeUninit<ScreenCharacter>; BUFFER_WIDTH]; BUFFER_HEIGHT];

#[repr(C)]
struct VGABuffer {
    buffer: Buffer,
    row_pos: usize,
    col_pos: usize,
}

#[repr(C, align(2))]
struct ScreenCharacter {
    ascii_char: u8,
    attribute: u8,
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
enum VGAError {
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
    
    fn new<FG, BG, BL>
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

impl VGABuffer {

    fn new() -> Self {
        Self {
            buffer: [[MaybeUninit::uninit(); BUFFER_WIDTH]; BUFFER_HEIGHT],
            row_pos: 0,
            col_pos: 0,
        }
    }

    fn write_char_to_buf(&mut self, char: ScreenCharacter) -> Result<(), VGAError> {
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
                        as *mut MaybeUninit<ScreenCharacter>;
                    core::ptr::write(char_ptr, MaybeUninit::new(char));
                    self.col_pos += 1;
                }
                Ok(())
            }
        } else {
            Err(VGAError::WriteError)
        }
    }

    pub fn write_plain_text_to_buf(&mut self, text: &str) -> Result<(), VGAError> {
        if !text.is_ascii() {
            Err(VGAError::InvalidASCIIError)
        } else {
            for ch in text.chars() {
                let ascii_ch = unsafe { core::ascii::Char::from_u8_unchecked(ch as u8) };
                let screen_ch = ScreenCharacter::new(ascii_ch, None, None, None);
                self.write_char_to_buf(screen_ch)?;
            }
            Ok(())
        }
    }

    pub fn write_fmt_text_to_buf(
        &mut self, 
        text: &str, 
        fg_color: ForegroundColor, 
        bg_color:BackgroundColor, 
        blink: bool) 
    -> Result<(), VGAError> {
        if !text.is_ascii() {
            Err(VGAError::InvalidASCIIError)
        } else {
            for ch in text.chars() {
                let ascii_ch = unsafe { core::ascii::Char::from_u8_unchecked(ch as u8) };
                let screen_ch = ScreenCharacter::new(
                    ascii_ch, 
                    fg_color, 
                    bg_color,
                    blink
                );
                self.write_char_to_buf(screen_ch)?;
            }
            Ok(())
        }
    }

    pub unsafe fn flush(&self, frame_buf: &mut [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT]) {
    unsafe {
        for i in 0..BUFFER_HEIGHT {
            for j in 0..BUFFER_WIDTH {
                let src_ptr = self.buffer
                    .get_unchecked(i)
                    .get_unchecked(j)
                    .as_ptr() as *const u16;
                let dst_ptr = frame_buf
                    .get_unchecked_mut(i)
                    .get_unchecked_mut(j) 
                    as *mut u16;
                let value = core::ptr::read_volatile(srcl_ptr);
                core::ptr::write_volatile(dst_ptr, value);
            }
        }
    }
}


    pub fn clear(&mut self) {
        unsafe {
            for i in 0..BUFFER_HEIGHT {
                for j in 0..BUFFER_WIDTH {
                    let buf_ptr = self.buffer
                        .get_unchecked_mut(i)
                        .get_unchecked_mut(j) as *mut MaybeUninit<ScreenCharacter>;
                    core::ptr::write_volatile(
                        buf_ptr, 
                        MaybeUninit::new(ScreenCharacter { 
                            ascii_char: 0x00, 
                            attribute: 0x00, 
                        })
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