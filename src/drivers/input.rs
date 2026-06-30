use core::ascii::Char;
use core::ops::Add;

use crate::arch::i686::kbd;
use crate::arch::i686::kbd::Key::{self, KpStar};
use crate::drivers::input;
use crate::drivers::input::InputAction::{AddChar, Cancel, DelCharBack, Submit};

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_CAPACITY: usize = BUFFER_WIDTH * BUFFER_HEIGHT;

type CharBuffer = [[Char; BUFFER_WIDTH]; BUFFER_HEIGHT];

#[repr(C)]
pub struct InputBuffer {
    buffer: CharBuffer,
    row_pos: usize,
    col_pos: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputAction {
    AddChar(Char),
    DelCharFront,
    DelCharBack,
    Submit,
    Cancel,
}

pub struct Input {
    buffer: InputBuffer,
    cur_action: InputAction,
    cur_keypress: Key,
    is_shift: bool,
    is_ctrl: bool,
    is_alt: bool,
    caps_lock: bool,
    num_lock: bool,
    scrl_lock: bool,
}

pub enum InputError {
    WriteError,
}

impl InputBuffer {
    
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

    /*fn del_char(&mut self) {

    }*/

    fn get_offset(&mut self) {
        (self.row_pos * BUFFER_WIDTH) + self.col_pos
    }

}

impl Input {

    pub fn new() -> Self {
        Self {
            buffer: InputBuffer { 
                buffer: [[Char::Null; BUFFER_WIDTH]; BUFFER_HEIGHT],
                row_pos: 0,
                col_pos: 0,
            },
            cur_action: None,
            cur_keypress: Key::default(),
            is_shift: false,
            is_ctrl: false,
            is_bool: false,
            caps_lock: false,
            num_lock: false,
            scrl_lock: false,
        }
    }

    pub fn is_shift(&mut self, state: bool) { self.is_shift = state; }
    pub fn is_ctrl(&mut self, state: bool) { self.is_ctrl = state; }
    pub fn is_alt(&mut self, state: bool) { self.is_alt = state; }
    pub fn is_capslk(&mut self, state: bool) { self.caps_lock = state; }
    pub fn is_numlk(&mut self, state: bool) { self.num_lock = state; }
    pub fn is_scrllk(&mut self, state: bool) { self.scrl_lock = state; }

    pub fn update_action(&mut self, key: Key) {
        let cur_char_res = map_key_to_char(key);
        if let Some(cur_char) = cur_char_res {
            let updated_char_res = 
            match (self.is_shift, self.caps_lock) {
                (false, false) => { None },
                (true, _) => { map_shift(cur_char) }, // shift + caps to be separated later
                (true, true) => { map_caps(cur_char) }
            }
            if let Some(updated_char) = updated_char_res {
                self.cur_action = AddChar(updated_char);
            } else {
                self.cur_action = AddChar(cur_char);
            }
        } else {
            match key {
                Key::Bksp => { self.cur_action = DelCharBack; },
                //Key:: => { self.cur_action = DelCharBack; },      // to be added later
                Key::Enter => { self.cur_action = Submit; },
                Key::Esc => { self.cur_action = Cancel; },
                _ => {}
            }
        }
    }

    pub fn execute_action(&self) {
        match self.cur_action {
            AddChar(ch) => self.buffer.write_char(ch),
            _ => {} // to be added later
        }
    }

}

fn map_key_to_char(key: Key) -> Option<Char> {
    match key {
        //alphabets
        Key::A => Some(Char::SmallA),
        Key::B => Some(Char::SmallB),
        Key::C => Some(Char::SmallC),
        Key::D => Some(Char::SmallD),
        Key::E => Some(Char::SmallE),
        Key::F => Some(Char::SmallF),
        Key::G => Some(Char::SmallG),
        Key::H => Some(Char::SmallH),
        Key::I => Some(Char::SmallI),
        Key::J => Some(Char::SmallJ),
        Key::K => Some(Char::SmallK),
        Key::L => Some(Char::SmallL),
        Key::M => Some(Char::SmallM),
        Key::N => Some(Char::SmallN),
        Key::O => Some(Char::SmallO),
        Key::P => Some(Char::SmallP),
        Key::Q => Some(Char::SmallQ),
        Key::R => Some(Char::SmallR),
        Key::S => Some(Char::SmallS),
        Key::T => Some(Char::SmallT),
        Key::U => Some(Char::SmallU),
        Key::V => Some(Char::SmallV),
        Key::W => Some(Char::SmallW),
        Key::X => Some(Char::SmallX),
        Key::Y => Some(Char::SmallY),
        Key::Z => Some(Char::SmallZ),

        //numbers
        Key::Num0 => Some(Char::Digit0),
        Key::Num1 => Some(Char::Digit1),
        Key::Num2 => Some(Char::Digit2),
        Key::Num3 => Some(Char::Digit3),
        Key::Num4 => Some(Char::Digit4),
        Key::Num5 => Some(Char::Digit5),
        Key::Num6 => Some(Char::Digit6),
        Key::Num7 => Some(Char::Digit7),
        Key::Num8 => Some(Char::Digit8),
        Key::Num9 => Some(Char::Digit9),

        //symbols
        Key::Minus => Some(Char::HyphenMinus),
        Key::Equal => Some(Char::EqualsSign),
        Key::OpSqBk => Some(Char::LeftSquareBracket),
        Key::ClSqBk => Some(Char::RightSquareBracket),
        Key::Smcln => Some(Char::Semicolon),
        Key::Quote => Some(Char::Apostrophe),
        Key::BkTk => Some(Char::GraveAccent),
        Key::FSlash => Some(Char::Solidus),
        Key::BSlash => Some(Char::ReverseSolidus),
        Key::Comma => Some(Char::Comma),
        Key::Period => Some(Char::FullStop),
        
        //space
        Key::Space => Some(Char::Space),

        //keypad (change later for only number lock)
        Key::KpStar => Some(Char::Asterisk),
        Key::KpMinus => Some(Char::HyphenMinus),
        Key::KpPlus => Some(Char::PlusSign),
        Key::KpPeriod => Some(Char::FullStop),
        Key::Kp0 => Some(Char::Digit0),
        Key::Kp1 => Some(Char::Digit1),
        Key::Kp2 => Some(Char::Digit2),
        Key::Kp3 => Some(Char::Digit3),
        Key::Kp4 => Some(Char::Digit4),
        Key::Kp5 => Some(Char::Digit5),
        Key::Kp6 => Some(Char::Digit6),
        Key::Kp7 => Some(Char::Digit7),
        Key::Kp8 => Some(Char::Digit8),
        Key::Kp9 => Some(Char::Digit9),

        //unmapped keys
        //_ => None,
    }
}

fn map_shift(input: Char) -> Option<Char> {
    match input {
        //alphabets
        Char::SmallA => Some(Char::CapitalA),
        Char::SmallB => Some(Char::CapitalB),
        Char::SmallC => Some(Char::CapitalC),
        Char::SmallD => Some(Char::CapitalD),
        Char::SmallE => Some(Char::CapitalE),
        Char::SmallF => Some(Char::CapitalF),
        Char::SmallG => Some(Char::CapitalG),
        Char::SmallH => Some(Char::CapitalH),
        Char::SmallI => Some(Char::CapitalI),
        Char::SmallJ => Some(Char::CapitalJ),
        Char::SmallK => Some(Char::CapitalK),
        Char::SmallL => Some(Char::CapitalL),
        Char::SmallM => Some(Char::CapitalM),
        Char::SmallN => Some(Char::CapitalN),
        Char::SmallO => Some(Char::CapitalO),
        Char::SmallP => Some(Char::CapitalP),
        Char::SmallQ => Some(Char::CapitalQ),
        Char::SmallR => Some(Char::CapitalR),
        Char::SmallS => Some(Char::CapitalS),
        Char::SmallT => Some(Char::CapitalT),
        Char::SmallU => Some(Char::CapitalU),
        Char::SmallV => Some(Char::CapitalV),
        Char::SmallW => Some(Char::CapitalW),
        Char::SmallX => Some(Char::CapitalX),
        Char::SmallY => Some(Char::CapitalY),
        Char::SmallZ => Some(Char::CapitalZ),

        //numbers
        Char::Digit1 => Some(Char::ExclamationMark),
        Char::Digit2 => Some(Char::QuotationMark),
        Char::Digit3 => Some(Char::NumberSign),
        Char::Digit4 => Some(Char::DollarSign),
        Char::Digit5 => Some(Char::PercentSign),
        Char::Digit6 => Some(Char::CircumflexAccent),
        Char::Digit7 => Some(Char::Ampersand),
        Char::Digit8 => Some(Char::Asterisk),
        Char::Digit9 => Some(Char::LeftParenthesis),
        Char::Digit0 => Some(Char::RightParenthesis),

        //symbols
        Char::HyphenMinus => Some(Char::LowLine),
        Char::EqualsSign => Some(Char::PlusSign),
        Char::LeftSquareBracket => Some(Char::LeftCurlyBracket),
        Char::RightSquareBracket => Some(Char::RightCurlyBracket),
        Char::Semicolon => Some(Char::Colon),
        Char::Apostrophe => Some(Char::QuotationMark),
        Char::GraveAccent => Some(Char::Tilde),
        Char::Solidus => Some(Char::QuestionMark),
        Char::ReverseSolidus => Some(Char::VerticalLine),
        Char::Comma => Some(Char::LessThanSign),
        Char::FullStop => Some(Char::GreaterThanSign),
        
        //space
        Char::Space => Some(Char::Space),

        _ => None,
    }
}

fn map_caps(input: Char) -> Option<Char> {
    match input {
        //alphabets
        Char::SmallA => Some(Char::CapitalA),
        Char::SmallB => Some(Char::CapitalB),
        Char::SmallC => Some(Char::CapitalC),
        Char::SmallD => Some(Char::CapitalD),
        Char::SmallE => Some(Char::CapitalE),
        Char::SmallF => Some(Char::CapitalF),
        Char::SmallG => Some(Char::CapitalG),
        Char::SmallH => Some(Char::CapitalH),
        Char::SmallI => Some(Char::CapitalI),
        Char::SmallJ => Some(Char::CapitalJ),
        Char::SmallK => Some(Char::CapitalK),
        Char::SmallL => Some(Char::CapitalL),
        Char::SmallM => Some(Char::CapitalM),
        Char::SmallN => Some(Char::CapitalN),
        Char::SmallO => Some(Char::CapitalO),
        Char::SmallP => Some(Char::CapitalP),
        Char::SmallQ => Some(Char::CapitalQ),
        Char::SmallR => Some(Char::CapitalR),
        Char::SmallS => Some(Char::CapitalS),
        Char::SmallT => Some(Char::CapitalT),
        Char::SmallU => Some(Char::CapitalU),
        Char::SmallV => Some(Char::CapitalV),
        Char::SmallW => Some(Char::CapitalW),
        Char::SmallX => Some(Char::CapitalX),
        Char::SmallY => Some(Char::CapitalY),
        Char::SmallZ => Some(Char::CapitalZ),
    }
}
