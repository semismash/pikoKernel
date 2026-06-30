use core::ascii::Char;

use crate::arch::i686::kbd;
use crate::arch::i686::kbd::Key;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

type CharBuffer = [[Char; BUFFER_WIDTH]; BUFFER_HEIGHT];

#[repr(C)]
pub struct InputBuffer {
    buffer: CharBuffer,
    row_pos: usize,
    col_pos: usize,
}

pub enum InputAction {
    None,
    AddChar(Char),
    DelChar,
    Submit,
    Cancel,
}

pub struct KeyStroke {
    active_action: InputAction,
}

/*
macro_rules! set_keystroke {
    ($output:expr, $($arg:expr),+ ) => {
        {
            $(
                if !(chk_pressed($arg)) return None;
            )+
            Some($output)
        }
    }
}
*/



