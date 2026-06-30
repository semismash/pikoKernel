use core::ascii::Char;

use crate::drivers::keyboard;
use crate::drivers::keyboard::Key;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

#[repr(C)]
pub struct InputBuffer {
    buffer: CharBuffer,
    row_pos: usize,
    col_pos: usize,
}

pub struct KeyStroke;

macro_rules! set_keystroke {
    ($output:expr, ) => {
        {
            
        }
    }
}



