use core::ascii::Char;

use crate::arch::i686::kbd::{self, Keyboard, Key, KEYPRESS_STACK_LENGTH};
use crate::arch::i686::kbd::KeyPress;
use crate::drivers::input;
use crate::drivers::input::InputAction::{AddChar, Cancel, DelChar, BackChar, Submit};
use core::sync::atomic::Ordering;

pub const BUFFER_LENGTH: usize = 256;

const KEYSTROKE_MAX_COUNT: usize = 256;
const KEYSTROKE_CAPACITY: usize = 8;   //max 8 keystrokes per keystroke, implemented by software, practically will never reach this high

//compile time check to make sure keystroke capacity does not exceed stack size
const _: u8 = [0][((KEYSTROKE_MAX_COUNT <= kbd::KEYPRESS_STACK_LENGTH as usize) as usize)];

type CharBuffer = [Char; BUFFER_LENGTH];
#[repr(C)]
pub struct InputBuffer {
    pub buffer: CharBuffer,
    pub idx: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputAction {
    None,
    AddChar(Char),
    DelChar,
    BackChar,
    Submit,
    Cancel,
}

pub enum InputError {
    WriteError,
}

impl InputBuffer {

    pub const fn new() -> Self {
        Self {
            buffer: [Char::Null; BUFFER_LENGTH],
            idx: 0,
        }
    }

    pub fn execute_action(&mut self, action: InputAction) -> Result<(), InputError> {
        match action {
            InputAction::None => { },
            InputAction::AddChar(ch) => { self.write_char(ch)?; },
            InputAction::DelChar => { self.del_char(); },
            InputAction::BackChar => { self.back_char(); },
            InputAction::Submit => { self.new_line(); },
            InputAction::Cancel => { self.clear_buffer(); },
        }
        Ok(())
    }

}

//buffer actions
impl InputBuffer {

    pub fn clear_buffer (&mut self) {
        unsafe {
            let buf_ptr = self.buffer.as_mut_ptr(); 
            core::ptr::write_bytes(buf_ptr, 0x00, BUFFER_LENGTH);   //Char::Null = 0x00
        }
        self.idx = 0;
    }
    
    pub fn write_char(&mut self, ch: Char) -> Result<(), InputError> {  
        //currently, directly changes the character that idx points to
        //needs to be changed between insert mode and add mode, the latter will move the remaining text in the buffer up
        if self.idx < BUFFER_LENGTH {
            unsafe {
                let mut idx_ptr = &mut self.buffer[self.idx] as *mut Char;
                core::ptr::write(idx_ptr, ch);
                self.idx += 1;
            }
            Ok(())
        } else {
            Err(InputError::WriteError)
        }
    }

    pub fn back_char(&mut self) {
        if (self.idx > 0) {
            unsafe {
                let src_ptr: *const Char = &self.buffer[self.idx] as *const Char;
                let dest_ptr: *mut Char = &mut self.buffer[self.idx - 1] as *mut Char;
                core::ptr::copy(src_ptr, dest_ptr, BUFFER_LENGTH - self.idx);
                self.buffer[BUFFER_LENGTH - 1] = Char::Null; // set final slot to null
                self.idx -= 1;
            }
        }
    }

    pub fn del_char(&mut self) {
        if (self.idx < BUFFER_LENGTH - 1) { 
            unsafe {
                let idx_ptr = &mut self.buffer[self.idx] as *mut Char;
                core::ptr::copy(idx_ptr.add(1), idx_ptr, BUFFER_LENGTH - self.idx - 1);
            }
        }
    }

    pub fn new_line(&mut self) -> Result<(), InputError> {
        self.write_char(Char::LineFeed)
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct KeyPressConfig {
    keypress_data: u16,
}
type KP = KeyPressConfig;
impl KeyPressConfig {

    const fn new(
        keycode: Key,
        extended: bool,
    ) -> Self {
        Self {
            keypress_data: (keycode as u16) | ((extended as u16) << 8),
        }
    }

    const fn default() -> Self {
        Self { keypress_data: 0x0000 }
    }

    pub fn equals_key(&self, other: &KeyPress) -> bool {
        self.keypress_data == other.keypress_data.load(Ordering::Relaxed)
    }

}

// NEED TO FIX THIS FUNCTION
pub fn get_action(keypress_stack: &[KeyPress; KEYPRESS_STACK_LENGTH as usize], active_stack_size: u8) -> InputAction {
    let mut bitmask: [u64; 4] = [0xFFFFFFFFFFFFFFFF; 4];  // 256 bits, one per keystroke entry, can be changed later
    //let mut keypress_stack_ptr: u8 = 0; //the last keystroke that was a valid candidate
    let mut candidate: usize = 0;   
    let mut final_candidate: Option<usize> = None;
    let mut single_key_fallback_idx: Option<usize> = None; 
    // ^^^ optimization, if cell matches and the place after that is padded with no key, then that MUST be the candidate 
    // (NOTE: requires PROPER initialization of keybinds in the array to work properly)
    for i in 0..KEYSTROKE_CAPACITY {    // scan each keypress row first, starting at key 1
        candidate = 0;
        let mut candidate_count: usize = 0;      //number of potential candidates for keypress 
        for j in 0..KEYSTROKE_MAX_COUNT {   // go through each individual keypress and check if it matches
            let cur_keypress = (KEYSTROKE_TABLE[j].1)[i];
            let word = j / 64;
            let bit = j % 64;

            // only checks on i = 0 i.e. first column
            if i == 0 && cur_keypress.equals_key(&keypress_stack[(active_stack_size - 1) as usize]) {
                if (KEYSTROKE_TABLE[j].1)[1].keypress_data == 0x0000 {
                    single_key_fallback_idx = Some(j);
                }
            }

            // normal check
            if (bitmask[word] >> bit & 0x1) == 1 {
                if cur_keypress.equals_key(&keypress_stack[i]) {
                    candidate = j;
                    candidate_count += 1;
                } else {
                    bitmask[word] &= !(1u64 << bit);  // turn bit j off if not valid
                }
            }
        }
        match candidate_count {
            0 => {  // reached end/ambiguous, exit early with default action None
                final_candidate = None;
                break;
            },    
            1 => {  // if 1 matches, we mark it as the correct one
                final_candidate = Some(candidate); 
            },  
            _ => {},    // reached end/ambiguous, exit early with default action None
        }
    }

    //revalidate multi-key keybind to prevent bugs
    if let Some(idx) = final_candidate {
        let mut i = 0;
        while i < KEYSTROKE_CAPACITY && (KEYSTROKE_TABLE[idx].1)[i].keypress_data != 0x0000 {   //check that its NOT none
            i += 1;
        }
        if (i == (active_stack_size as usize)) {    // if i equals stack size, the correct keys are beingg pressed
            return (KEYSTROKE_TABLE[idx].0).match_key_stroke_to_action();
        }
    }

    if let Some(idx) = single_key_fallback_idx {    // if nothing matches, default to the most recently pressed key
        return (KEYSTROKE_TABLE[idx].0).match_key_stroke_to_action();
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

#[derive(Debug, Clone, Copy)]
struct KeyStrokeMacroInputRow { keystroke: KeyStroke, keypresses: [KP; KEYSTROKE_CAPACITY] }

macro_rules! create_keystroke_table {
    ($($keystroke:expr => [$($scancode:expr),*]),* $(,)?) => {
        {
            let mut inputs = [KeyStrokeMacroInputRow {
                keystroke: KeyStroke::default(),
                keypresses: [KP::default(); KEYSTROKE_CAPACITY],
            }; KEYSTROKE_MAX_COUNT];
            
            let mut idx = 0;
            $(
                inputs[idx] = KeyStrokeMacroInputRow {
                    keystroke: $keystroke,
                    keypresses: pad_keypresses(&[$($scancode),*]),
                };
                idx += 1;
            )*
            create_keystroke_table(inputs)
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyStroke {
    //list of keystrokes
    
    None,
    
    // lowercase Characters
    PutCSmallA, PutCSmallB, PutCSmallC, PutCSmallD, PutCSmallE,
    PutCSmallF, PutCSmallG, PutCSmallH, PutCSmallI, PutCSmallJ,
    PutCSmallK, PutCSmallL, PutCSmallM, PutCSmallN, PutCSmallO,
    PutCSmallP, PutCSmallQ, PutCSmallR, PutCSmallS, PutCSmallT,
    PutCSmallU, PutCSmallV, PutCSmallW, PutCSmallX, PutCSmallY,
    PutCSmallZ,

    // UPPERCASE CHARACTERS
    PutCBigA, PutCBigB, PutCBigC, PutCBigD, PutCBigE,
    PutCBigF, PutCBigG, PutCBigH, PutCBigI, PutCBigJ,
    PutCBigK, PutCBigL, PutCBigM, PutCBigN, PutCBigO,
    PutCBigP, PutCBigQ, PutCBigR, PutCBigS, PutCBigT,
    PutCBigU, PutCBigV, PutCBigW, PutCBigX, PutCBigY,
    PutCBigZ,

    // numbers
    PutCNum1, PutCNum2, PutCNum3, PutCNum4, PutCNum5,
    PutCNum6, PutCNum7, PutCNum8, PutCNum9, PutCNum0,

    // base punctuations
    PutCMinus, PutCEqual, PutCOpSqBk, PutCClSqBk, PutCSmcln,
    PutCQuote, PutCBkTk, PutCBSlash, PutCComma, PutCPeriod, PutCFSlash,

    // shifted punctuations
    PutCUnderscore, PutCPlus, PutCOpCuBk, PutCClCuBk, PutCColon,
    PutCDoubleQuote, PutCTilde, PutCPipe, PutCLessThan, PutCGreaterThan, PutCQuestion,

    // shifted numrow
    PutCExclamation, PutCAt, PutCHash, PutCDollar, PutCPercent,
    PutCCaret, PutCAmpersand, PutCAsterisk, PutCOpenParen, PutCCloseParen,

    // other
    Space,
    Backspace,
    Delete,
    Enter,
    Cancel,
}

type KS = KeyStroke;

impl KeyStroke {

    const fn default() -> Self {
        Self::None
    }

    const fn match_key_stroke_to_action(&self) -> InputAction {
        match self {
            // lowercase
            KS::PutCSmallA => InputAction::AddChar(Char::SmallA),
            KS::PutCSmallB => InputAction::AddChar(Char::SmallB),
            KS::PutCSmallC => InputAction::AddChar(Char::SmallC),
            KS::PutCSmallD => InputAction::AddChar(Char::SmallD),
            KS::PutCSmallE => InputAction::AddChar(Char::SmallE),
            KS::PutCSmallF => InputAction::AddChar(Char::SmallF),
            KS::PutCSmallG => InputAction::AddChar(Char::SmallG),
            KS::PutCSmallH => InputAction::AddChar(Char::SmallH),
            KS::PutCSmallI => InputAction::AddChar(Char::SmallI),
            KS::PutCSmallJ => InputAction::AddChar(Char::SmallJ),
            KS::PutCSmallK => InputAction::AddChar(Char::SmallK),
            KS::PutCSmallL => InputAction::AddChar(Char::SmallL),
            KS::PutCSmallM => InputAction::AddChar(Char::SmallM),
            KS::PutCSmallN => InputAction::AddChar(Char::SmallN),
            KS::PutCSmallO => InputAction::AddChar(Char::SmallO),
            KS::PutCSmallP => InputAction::AddChar(Char::SmallP),
            KS::PutCSmallQ => InputAction::AddChar(Char::SmallQ),
            KS::PutCSmallR => InputAction::AddChar(Char::SmallR),
            KS::PutCSmallS => InputAction::AddChar(Char::SmallS),
            KS::PutCSmallT => InputAction::AddChar(Char::SmallT),
            KS::PutCSmallU => InputAction::AddChar(Char::SmallU),
            KS::PutCSmallV => InputAction::AddChar(Char::SmallV),
            KS::PutCSmallW => InputAction::AddChar(Char::SmallW),
            KS::PutCSmallX => InputAction::AddChar(Char::SmallX),
            KS::PutCSmallY => InputAction::AddChar(Char::SmallY),
            KS::PutCSmallZ => InputAction::AddChar(Char::SmallZ),

            // uppercase
            KS::PutCBigA => InputAction::AddChar(Char::CapitalA),
            KS::PutCBigB => InputAction::AddChar(Char::CapitalB),
            KS::PutCBigC => InputAction::AddChar(Char::CapitalC),
            KS::PutCBigD => InputAction::AddChar(Char::CapitalD),
            KS::PutCBigE => InputAction::AddChar(Char::CapitalE),
            KS::PutCBigF => InputAction::AddChar(Char::CapitalF),
            KS::PutCBigG => InputAction::AddChar(Char::CapitalG),
            KS::PutCBigH => InputAction::AddChar(Char::CapitalH),
            KS::PutCBigI => InputAction::AddChar(Char::CapitalI),
            KS::PutCBigJ => InputAction::AddChar(Char::CapitalJ),
            KS::PutCBigK => InputAction::AddChar(Char::CapitalK),
            KS::PutCBigL => InputAction::AddChar(Char::CapitalL),
            KS::PutCBigM => InputAction::AddChar(Char::CapitalM),
            KS::PutCBigN => InputAction::AddChar(Char::CapitalN),
            KS::PutCBigO => InputAction::AddChar(Char::CapitalO),
            KS::PutCBigP => InputAction::AddChar(Char::CapitalP),
            KS::PutCBigQ => InputAction::AddChar(Char::CapitalQ),
            KS::PutCBigR => InputAction::AddChar(Char::CapitalR),
            KS::PutCBigS => InputAction::AddChar(Char::CapitalS),
            KS::PutCBigT => InputAction::AddChar(Char::CapitalT),
            KS::PutCBigU => InputAction::AddChar(Char::CapitalU),
            KS::PutCBigV => InputAction::AddChar(Char::CapitalV),
            KS::PutCBigW => InputAction::AddChar(Char::CapitalW),
            KS::PutCBigX => InputAction::AddChar(Char::CapitalX),
            KS::PutCBigY => InputAction::AddChar(Char::CapitalY),
            KS::PutCBigZ => InputAction::AddChar(Char::CapitalZ),

            // numbers
            KS::PutCNum1 => InputAction::AddChar(Char::Digit1),
            KS::PutCNum2 => InputAction::AddChar(Char::Digit2),
            KS::PutCNum3 => InputAction::AddChar(Char::Digit3),
            KS::PutCNum4 => InputAction::AddChar(Char::Digit4),
            KS::PutCNum5 => InputAction::AddChar(Char::Digit5),
            KS::PutCNum6 => InputAction::AddChar(Char::Digit6),
            KS::PutCNum7 => InputAction::AddChar(Char::Digit7),
            KS::PutCNum8 => InputAction::AddChar(Char::Digit8),
            KS::PutCNum9 => InputAction::AddChar(Char::Digit9),
            KS::PutCNum0 => InputAction::AddChar(Char::Digit0),

            //shifted numbers
            KS::PutCExclamation => InputAction::AddChar(Char::ExclamationMark),
            KS::PutCAt          => InputAction::AddChar(Char::CommercialAt),
            KS::PutCHash        => InputAction::AddChar(Char::NumberSign), // or Char::NumberSign
            KS::PutCDollar      => InputAction::AddChar(Char::DollarSign),
            KS::PutCPercent     => InputAction::AddChar(Char::PercentSign),
            KS::PutCCaret       => InputAction::AddChar(Char::CircumflexAccent),
            KS::PutCAmpersand   => InputAction::AddChar(Char::Ampersand),
            KS::PutCAsterisk    => InputAction::AddChar(Char::Asterisk),
            KS::PutCOpenParen   => InputAction::AddChar(Char::LeftParenthesis),
            KS::PutCCloseParen  => InputAction::AddChar(Char::RightParenthesis),

            // base Punctuations
            KS::PutCMinus      => InputAction::AddChar(Char::HyphenMinus), 
            KS::PutCEqual      => InputAction::AddChar(Char::EqualsSign),
            KS::PutCOpSqBk     => InputAction::AddChar(Char::LeftSquareBracket),
            KS::PutCClSqBk     => InputAction::AddChar(Char::RightSquareBracket),
            KS::PutCSmcln      => InputAction::AddChar(Char::Semicolon),
            KS::PutCQuote      => InputAction::AddChar(Char::Apostrophe),
            KS::PutCBkTk       => InputAction::AddChar(Char::GraveAccent),
            KS::PutCBSlash     => InputAction::AddChar(Char::ReverseSolidus),
            KS::PutCComma      => InputAction::AddChar(Char::Comma),
            KS::PutCPeriod     => InputAction::AddChar(Char::FullStop),
            KS::PutCFSlash     => InputAction::AddChar(Char::Solidus),

            // Shifted Punctuations
            KS::PutCUnderscore  => InputAction::AddChar(Char::LowLine), // or Char::Underscore
            KS::PutCPlus        => InputAction::AddChar(Char::PlusSign),
            KS::PutCOpCuBk      => InputAction::AddChar(Char::LeftCurlyBracket),
            KS::PutCClCuBk      => InputAction::AddChar(Char::RightCurlyBracket),
            KS::PutCColon       => InputAction::AddChar(Char::Colon),
            KS::PutCDoubleQuote => InputAction::AddChar(Char::QuotationMark),
            KS::PutCTilde       => InputAction::AddChar(Char::Tilde),
            KS::PutCPipe        => InputAction::AddChar(Char::VerticalLine), // or Char::Pipe
            KS::PutCLessThan    => InputAction::AddChar(Char::GreaterThanSign),
            KS::PutCGreaterThan => InputAction::AddChar(Char::LessThanSign),
            KS::PutCQuestion    => InputAction::AddChar(Char::QuestionMark),

            // Controls
            /*
            KS::Space     => InputAction::AddChar(Char::Space),
            KS::Backspace => InputAction::BackChar,
            KS::Delete    => InputAction::DelChar,
            KS::Enter     => InputAction::Submit,
            KS::Cancel    => InputAction::Cancel,
            */
            KS::Space     => InputAction::AddChar(Char::Space),
            KS::Backspace => InputAction::BackChar,
            KS::Delete    => InputAction::None,
            KS::Enter     => InputAction::Submit,
            KS::Cancel    => InputAction::None,

        _ => InputAction::None,
        }
    }
}

type KeyStrokeEntry = (KeyStroke, [KP; KEYSTROKE_CAPACITY]);

static KEYSTROKE_TABLE: [KeyStrokeEntry; KEYSTROKE_MAX_COUNT] = create_keystroke_table!(
    KS::None => [],

    // lowercase
    KS::PutCSmallA => [KP::new(Key::A, false)],
    KS::PutCSmallB => [KP::new(Key::B, false)],
    KS::PutCSmallC => [KP::new(Key::C, false)],
    KS::PutCSmallD => [KP::new(Key::D, false)],
    KS::PutCSmallE => [KP::new(Key::E, false)],
    KS::PutCSmallF => [KP::new(Key::F, false)],
    KS::PutCSmallG => [KP::new(Key::G, false)],
    KS::PutCSmallH => [KP::new(Key::H, false)],
    KS::PutCSmallI => [KP::new(Key::I, false)],
    KS::PutCSmallJ => [KP::new(Key::J, false)],
    KS::PutCSmallK => [KP::new(Key::K, false)],
    KS::PutCSmallL => [KP::new(Key::L, false)],
    KS::PutCSmallM => [KP::new(Key::M, false)],
    KS::PutCSmallN => [KP::new(Key::N, false)],
    KS::PutCSmallO => [KP::new(Key::O, false)],
    KS::PutCSmallP => [KP::new(Key::P, false)],
    KS::PutCSmallQ => [KP::new(Key::Q, false)],
    KS::PutCSmallR => [KP::new(Key::R, false)],
    KS::PutCSmallS => [KP::new(Key::S, false)],
    KS::PutCSmallT => [KP::new(Key::T, false)],
    KS::PutCSmallU => [KP::new(Key::U, false)],
    KS::PutCSmallV => [KP::new(Key::V, false)],
    KS::PutCSmallW => [KP::new(Key::W, false)],
    KS::PutCSmallX => [KP::new(Key::X, false)],
    KS::PutCSmallY => [KP::new(Key::Y, false)],
    KS::PutCSmallZ => [KP::new(Key::Z, false)],

    // uppercase
    KS::PutCBigA => [KP::new(Key::LShift, false), KP::new(Key::A, false)],
    KS::PutCBigB => [KP::new(Key::LShift, false), KP::new(Key::B, false)],
    KS::PutCBigC => [KP::new(Key::LShift, false), KP::new(Key::C, false)],
    KS::PutCBigD => [KP::new(Key::LShift, false), KP::new(Key::D, false)],
    KS::PutCBigE => [KP::new(Key::LShift, false), KP::new(Key::E, false)],
    KS::PutCBigF => [KP::new(Key::LShift, false), KP::new(Key::F, false)],
    KS::PutCBigG => [KP::new(Key::LShift, false), KP::new(Key::G, false)],
    KS::PutCBigH => [KP::new(Key::LShift, false), KP::new(Key::H, false)],
    KS::PutCBigI => [KP::new(Key::LShift, false), KP::new(Key::I, false)],
    KS::PutCBigJ => [KP::new(Key::LShift, false), KP::new(Key::J, false)],
    KS::PutCBigK => [KP::new(Key::LShift, false), KP::new(Key::K, false)],
    KS::PutCBigL => [KP::new(Key::LShift, false), KP::new(Key::L, false)],
    KS::PutCBigM => [KP::new(Key::LShift, false), KP::new(Key::M, false)],
    KS::PutCBigN => [KP::new(Key::LShift, false), KP::new(Key::N, false)],
    KS::PutCBigO => [KP::new(Key::LShift, false), KP::new(Key::O, false)],
    KS::PutCBigP => [KP::new(Key::LShift, false), KP::new(Key::P, false)],
    KS::PutCBigQ => [KP::new(Key::LShift, false), KP::new(Key::Q, false)],
    KS::PutCBigR => [KP::new(Key::LShift, false), KP::new(Key::R, false)],
    KS::PutCBigS => [KP::new(Key::LShift, false), KP::new(Key::S, false)],
    KS::PutCBigT => [KP::new(Key::LShift, false), KP::new(Key::T, false)],
    KS::PutCBigU => [KP::new(Key::LShift, false), KP::new(Key::U, false)],
    KS::PutCBigV => [KP::new(Key::LShift, false), KP::new(Key::V, false)],
    KS::PutCBigW => [KP::new(Key::LShift, false), KP::new(Key::W, false)],
    KS::PutCBigX => [KP::new(Key::LShift, false), KP::new(Key::X, false)],
    KS::PutCBigY => [KP::new(Key::LShift, false), KP::new(Key::Y, false)],
    KS::PutCBigZ => [KP::new(Key::LShift, false), KP::new(Key::Z, false)],

    // uppercase shift
    /*KS::PutCBigA => [KP::new(Key::RShift, false), KP::new(Key::A, false)],
    KS::PutCBigB => [KP::new(Key::RShift, false), KP::new(Key::B, false)],
    KS::PutCBigC => [KP::new(Key::RShift, false), KP::new(Key::C, false)],
    KS::PutCBigD => [KP::new(Key::RShift, false), KP::new(Key::D, false)],
    KS::PutCBigE => [KP::new(Key::RShift, false), KP::new(Key::E, false)],
    KS::PutCBigF => [KP::new(Key::RShift, false), KP::new(Key::F, false)],
    KS::PutCBigG => [KP::new(Key::RShift, false), KP::new(Key::G, false)],
    KS::PutCBigH => [KP::new(Key::RShift, false), KP::new(Key::H, false)],
    KS::PutCBigI => [KP::new(Key::RShift, false), KP::new(Key::I, false)],
    KS::PutCBigJ => [KP::new(Key::RShift, false), KP::new(Key::J, false)],
    KS::PutCBigK => [KP::new(Key::RShift, false), KP::new(Key::K, false)],
    KS::PutCBigL => [KP::new(Key::RShift, false), KP::new(Key::L, false)],
    KS::PutCBigM => [KP::new(Key::RShift, false), KP::new(Key::M, false)],
    KS::PutCBigN => [KP::new(Key::RShift, false), KP::new(Key::N, false)],
    KS::PutCBigO => [KP::new(Key::RShift, false), KP::new(Key::O, false)],
    KS::PutCBigP => [KP::new(Key::RShift, false), KP::new(Key::P, false)],
    KS::PutCBigQ => [KP::new(Key::RShift, false), KP::new(Key::Q, false)],
    KS::PutCBigR => [KP::new(Key::RShift, false), KP::new(Key::R, false)],
    KS::PutCBigS => [KP::new(Key::RShift, false), KP::new(Key::S, false)],
    KS::PutCBigT => [KP::new(Key::RShift, false), KP::new(Key::T, false)],
    KS::PutCBigU => [KP::new(Key::RShift, false), KP::new(Key::U, false)],
    KS::PutCBigV => [KP::new(Key::RShift, false), KP::new(Key::V, false)],
    KS::PutCBigW => [KP::new(Key::RShift, false), KP::new(Key::W, false)],
    KS::PutCBigX => [KP::new(Key::RShift, false), KP::new(Key::X, false)],
    KS::PutCBigY => [KP::new(Key::RShift, false), KP::new(Key::Y, false)],
    KS::PutCBigZ => [KP::new(Key::RShift, false), KP::new(Key::Z, false)],*/

    // numbers
    KS::PutCNum1 => [KP::new(Key::Num1, false)],
    KS::PutCNum2 => [KP::new(Key::Num2, false)],
    KS::PutCNum3 => [KP::new(Key::Num3, false)],
    KS::PutCNum4 => [KP::new(Key::Num4, false)],
    KS::PutCNum5 => [KP::new(Key::Num5, false)],
    KS::PutCNum6 => [KP::new(Key::Num6, false)],
    KS::PutCNum7 => [KP::new(Key::Num7, false)],
    KS::PutCNum8 => [KP::new(Key::Num8, false)],
    KS::PutCNum9 => [KP::new(Key::Num9, false)],
    KS::PutCNum0 => [KP::new(Key::Num0, false)],

    // shifted numbers lsfhit
    KS::PutCExclamation => [KP::new(Key::LShift, false), KP::new(Key::Num1, false)],
    KS::PutCAt          => [KP::new(Key::LShift, false), KP::new(Key::Num2, false)],
    KS::PutCHash        => [KP::new(Key::LShift, false), KP::new(Key::Num3, false)],
    KS::PutCDollar      => [KP::new(Key::LShift, false), KP::new(Key::Num4, false)],
    KS::PutCPercent     => [KP::new(Key::LShift, false), KP::new(Key::Num5, false)],
    KS::PutCCaret       => [KP::new(Key::LShift, false), KP::new(Key::Num6, false)],
    KS::PutCAmpersand   => [KP::new(Key::LShift, false), KP::new(Key::Num7, false)],
    KS::PutCAsterisk    => [KP::new(Key::LShift, false), KP::new(Key::Num8, false)],
    KS::PutCOpenParen   => [KP::new(Key::LShift, false), KP::new(Key::Num9, false)],
    KS::PutCCloseParen  => [KP::new(Key::LShift, false), KP::new(Key::Num0, false)],

    // shifted numbers rshift
    /*KS::PutCExclamation => [KP::new(Key::RShift, false), KP::new(Key::Num1, false)],
    KS::PutCAt          => [KP::new(Key::RShift, false), KP::new(Key::Num2, false)],
    KS::PutCHash        => [KP::new(Key::RShift, false), KP::new(Key::Num3, false)],
    KS::PutCDollar      => [KP::new(Key::RShift, false), KP::new(Key::Num4, false)],
    KS::PutCPercent     => [KP::new(Key::RShift, false), KP::new(Key::Num5, false)],
    KS::PutCCaret       => [KP::new(Key::RShift, false), KP::new(Key::Num6, false)],
    KS::PutCAmpersand   => [KP::new(Key::RShift, false), KP::new(Key::Num7, false)],
    KS::PutCAsterisk    => [KP::new(Key::RShift, false), KP::new(Key::Num8, false)],
    KS::PutCOpenParen   => [KP::new(Key::RShift, false), KP::new(Key::Num9, false)],
    KS::PutCCloseParen  => [KP::new(Key::RShift, false), KP::new(Key::Num0, false)],*/


    // base punctions
    KS::PutCMinus      => [KP::new(Key::Minus, false)],
    KS::PutCEqual      => [KP::new(Key::Equal, false)],
    KS::PutCOpSqBk     => [KP::new(Key::OpSqBk, false)],
    KS::PutCClSqBk     => [KP::new(Key::ClSqBk, false)],
    KS::PutCSmcln      => [KP::new(Key::Smcln, false)],
    KS::PutCQuote      => [KP::new(Key::Quote, false)],
    KS::PutCBkTk       => [KP::new(Key::BkTk, false)],
    KS::PutCBSlash     => [KP::new(Key::BSlash, false)],
    KS::PutCComma      => [KP::new(Key::Comma, false)],
    KS::PutCPeriod     => [KP::new(Key::Period, false)],
    KS::PutCFSlash     => [KP::new(Key::FSlash, false)],

    // lshift punctionaion
    KS::PutCUnderscore  => [KP::new(Key::LShift, false), KP::new(Key::Minus, false)],
    KS::PutCPlus        => [KP::new(Key::LShift, false), KP::new(Key::Equal, false)],
    KS::PutCOpCuBk      => [KP::new(Key::LShift, false), KP::new(Key::OpSqBk, false)],
    KS::PutCClCuBk      => [KP::new(Key::LShift, false), KP::new(Key::ClSqBk, false)],
    KS::PutCColon       => [KP::new(Key::LShift, false), KP::new(Key::Smcln, false)],
    KS::PutCDoubleQuote => [KP::new(Key::LShift, false), KP::new(Key::Quote, false)],
    KS::PutCTilde       => [KP::new(Key::LShift, false), KP::new(Key::BkTk, false)],
    KS::PutCPipe        => [KP::new(Key::LShift, false), KP::new(Key::BSlash, false)],
    KS::PutCLessThan    => [KP::new(Key::LShift, false), KP::new(Key::Comma, false)],
    KS::PutCGreaterThan => [KP::new(Key::LShift, false), KP::new(Key::Period, false)],
    KS::PutCQuestion    => [KP::new(Key::LShift, false), KP::new(Key::FSlash, false)],

    // rshift punction
    /*KS::PutCUnderscore  => [KP::new(Key::RShift, false), KP::new(Key::Minus, false)],
    KS::PutCPlus        => [KP::new(Key::RShift, false), KP::new(Key::Equal, false)],
    KS::PutCOpCuBk      => [KP::new(Key::RShift, false), KP::new(Key::OpSqBk, false)],
    KS::PutCClCuBk      => [KP::new(Key::RShift, false), KP::new(Key::ClSqBk, false)],
    KS::PutCColon       => [KP::new(Key::RShift, false), KP::new(Key::Smcln, false)],
    KS::PutCDoubleQuote => [KP::new(Key::RShift, false), KP::new(Key::Quote, false)],
    KS::PutCTilde       => [KP::new(Key::RShift, false), KP::new(Key::BkTk, false)],
    KS::PutCPipe        => [KP::new(Key::RShift, false), KP::new(Key::BSlash, false)],
    KS::PutCLessThan    => [KP::new(Key::RShift, false), KP::new(Key::Comma, false)],
    KS::PutCGreaterThan => [KP::new(Key::RShift, false), KP::new(Key::Period, false)],
    KS::PutCQuestion    => [KP::new(Key::RShift, false), KP::new(Key::FSlash, false)],*/

    // other
    KS::Space     => [KP::new(Key::Space, false)],
    KS::Backspace => [KP::new(Key::Bksp, false)],
    KS::Delete    => [KP::new(Key::Tab, false)],
    KS::Enter     => [KP::new(Key::Enter, false)],
    KS::Cancel    => [KP::new(Key::Esc, false)],

    // debug
    KS::PutCBigZ  => [KP::new(Key::Num4, false), KP::new(Key::Num5, false), KP::new(Key::Num6, false)], //debuggun purpose
);
