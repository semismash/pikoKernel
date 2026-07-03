use core::ascii::Char;

use crate::arch::i686::kbd::{self, Keyboard, Key};
use crate::arch::i686::kbd::KeyPress as KP;
use crate::drivers::input;
use crate::drivers::input::InputAction::{AddChar, Cancel, DelCharBack, Submit};

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_CAPACITY: usize = BUFFER_WIDTH * BUFFER_HEIGHT;

const KEYSTROKE_MAX_COUNT: usize = 256;
const KEYSTROKE_CAPACITY: usize = 8;   //max 8 keystrokes per keystroke, implemented by software, practically will never reach this high

//compile time check to make sure keystroke capacity does not exceed stack size
const _: u8 = [0][(KEYSTROKE_MAX_COUNT >= kbd::KEYPRESS_STACK_LENGTH as usize)];

type KeyStrokeEntry = (KeyStroke, [KP; KEYSTROKE_CAPACITY]);
static KEYSTROKE_TABLE: [KeyStrokeEntry; KEYSTROKE_MAX_COUNT] = create_keystroke_table!(
    KS::None => [],
    KS::PutCSmallA => [KP::new(Key::A, false)],
    KS::PutCSmallB => [KP::new(Key::B, false)],
    KS::PutCSmallC => [KP::new(Key::B, false)],
);

type CharBuffer = [[Char; BUFFER_WIDTH]; BUFFER_HEIGHT];
#[repr(C)]
pub struct InputBuffer {
    buffer: CharBuffer,
    row_pos: usize,
    col_pos: usize,
}

enum EchoMode {

}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputAction {
    None,
    AddChar(Char),
    DelCharFront,
    DelCharBack,
    Submit,
    Cancel,
}

pub enum InputError {
    WriteError,
}

impl InputBuffer {

    fn initialize() -> Self {
        Self {
            buffer: [[Char::Null; BUFFER_WIDTH]; BUFFER_HEIGHT],
            row_pos: 0,
            col_pos: 0,
        }
    }

    fn clear_buffer (&mut self) {
        unsafe {
            for i in 0..BUFFER_HEIGHT {
                for j in 0..BUFFER_WIDTH {
                    let buf_ptr = self.buffer
                        .get_unchecked_mut(i)
                        .get_unchecked_mut(j) as *mut Char;
                    core::ptr::write(
                        buf_ptr, 
                        Char::Null,
                    );
                }
            }
        }
        self.row_pos = 0;
        self.col_pos = 0;
    }
    
    fn write_char(&mut self, ch: Char) -> Result<(), InputError> {
        if self.get_offset() < BUFFER_CAPACITY {
            unsafe {
                if self.col_pos >= BUFFER_WIDTH {
                    self.row_pos += 1;
                    self.col_pos = 0;
                }
                let char_ptr = self.buffer
                    .get_unchecked_mut(self.row_pos)
                    .get_unchecked_mut(self.col_pos)
                    as *mut Char;
                core::ptr::write(char_ptr, ch);
                self.col_pos += 1;
                Ok(())
            }
        } else {
            Err(InputError::WriteError)
        }
    }

    fn back_char(&mut self) {
        if (self.get_offset() > 0) {
            unsafe {
                let char_ptr = self.buffer
                    .get_unchecked_mut(self.row_pos)
                    .get_unchecked_mut(self.col_pos)
                    as *mut Char;
                core::ptr::copy(char_ptr, char_ptr.sub(1), BUFFER_CAPACITY - self.get_offset() - 1);
                char_ptr -= 1;
            }
        }
    }

    fn del_char(&mut self) {
        if (self.get_offset() < BUFFER_CAPACITY - 1) {
            unsafe {
                let char_ptr = self.buffer
                    .get_unchecked_mut(self.row_pos)
                    .get_unchecked_mut(self.col_pos)
                    as *mut Char;
                core::ptr::copy(char_ptr.add(1), char_ptr, BUFFER_CAPACITY - self.get_offset() - 1);
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_pos < BUFFER_HEIGHT {
            self.row_pos += 1;
            self.col_pos = 0;
        }
    }

    fn get_offset(&mut self) -> usize {
        (self.row_pos * BUFFER_WIDTH) + self.col_pos
    }

}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyStroke {
    //list of keystrokes
    #[default] None = 0x00u8,
    PutCSmallA,
    PutCSmallB,
    PutCSmallC,
}

type KS = KeyStroke;

impl KeyStroke {
    fn match_key_stroke_to_action(&self) -> InputAction {
        match self {
            KS::PutCSmallA => AddChar(Char::SmallA),
            KS::PutCSmallB => AddChar(Char::SmallB),
            KS::PutCSmallC => AddChar(Char::SmallC),

            //catch all
            _ => InputAction::None,
        }
    }
}

pub fn get_action(keypress_stack: &[KP; KEYPRESS_STACK_LENGTH]) -> InputAction {
    let mut bitmask: u64 = 0xFFFFFFFFFFFFFFFF;   //for 256 keystroke cap, can be changed later
    let mut keypress_stack_ptr: u8 = 0;
    let mut candidate: usize = 0;    //the last keystroke that was a valid candidate
    for i in 0..KEYSTROKE_CAPACITY {    // scan each keypress row first, starting at key 1
        candidate = 0;
        let mut candidate_count: usize = 0; //number of potential candidates for keypress 
        for j in 0..KEYSTROKE_MAX_COUNT {   // go through each individual keypress and check if it
            let cur_keypress = (KEYSTROKE_TABLE[j].1)[i];
            let cur_ptr = keypress_stack_ptr;
            if (bitmask >> i & 0x1) == 1 {
                if (cur_keypress.equals_key(keypress_stack[cur_ptr])) {
                    candidate = j;
                    candidate_count += 1;
                } else {
                    bitmask &= !(1 << i);   // turn bit off if not valid
                }
            }
        }
        match candidate_count {
            0 => { return InputAction::None; }, // reached end/ambiguous, exit early with default action None (TO BE CHANGEGD LATER TO CHECKING THE MOST RECENT KEYPRESS)
            1 => { return (KEYSTROKE_TABLE[candidate].0).match_key_stroke_to_action(); },   // if 1 matches, it is the correct one
            _ => {},    // ambiguous, ignore and go through loop as normal
        }
    }
    InputAction::None   // reached end/ambiguous, exit early with default action None
}

const fn create_keystroke_table(inputs: [KeyStrokeMacroInputRow; KEYSTROKE_MAX_COUNT]) -> [KeyStrokeEntry; KEYSTROKE_MAX_COUNT] {
    let mut table 
        = [(KeyStroke::default(), [KP::default(); KEYSTROKE_CAPACITY]); KEYSTROKE_MAX_COUNT];
    let mut i = 0;
    while i < KEYSTROKE_MAX_COUNT {
        table[i] = (inputs[i].keystroke, inputs[i].keypresses);
        i += 1;
    }
    table
}

const fn pad_keypresses(src: &[KP]) -> [KP; KEYSTROKE_CAPACITY] {   //helper function
    let mut dst = [KP::default(); KEYSTROKE_CAPACITY];
    let mut i = 0;
    while i < src.len() {
        dst[i] = src[i];
        i += 1;
    }
    dst
}

struct KeyStrokeMacroInputRow { keystroke: KeyStroke, keypresses: [KP; KEYSTROKE_CAPACITY] }

macro_rules! create_keystroke_table {
    ($($keystroke:expr => [$($scancode:expr),*]),* $(,)?) => {
        create_keystroke_table(
            [$(
                KeyStrokeMacroInputRow {
                    keystroke: $keystroke,
                    keypresses: pad_keypresses(&[$($scancode),*]),
                }
            ),*]
        )
    };
}