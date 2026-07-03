use core::sync::atomic::{AtomicU16, Ordering};
use core::ptr;

use crate::arch::i686::kbd::Key::S;

const RELEASE_BYTE: u8 = 0x80;
const EXTENDED_BYTE: u8 = 0xE0;

pub const KEYPRESS_STACK_LENGTH: u8 = 128;

static mut IS_EXTENDED: bool = false;
static mut KEYPRESS_STACK: [KeyPress; KEYPRESS_STACK_LENGTH] 
    = [KeyPress::default(); KEYPRESS_STACK_LENGTH];
static mut KEYPRESS_STACK_POINTER: u8 = 0;  // POSITION OF THE NEXT FREE SLOT

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Key {
    #[default] None = 0x00u8, 
    Esc = 0x01u8,
    Num1 = 0x02u8, Num2 = 0x03u8, Num3 = 0x04u8, Num4 = 0x05u8, Num5 = 0x06u8,
    Num6 = 0x07u8, Num7 = 0x08u8, Num8 = 0x09u8, Num9 = 0x0Au8, Num0 = 0x0Bu8,
    Minus = 0x0Cu8, Equal = 0x0Du8, Bksp = 0x0Eu8, Tab = 0x0Fu8,
    Q = 0x10u8, W = 0x11u8, E = 0x12u8, R = 0x13u8, T = 0x14u8,
    Y = 0x15u8, U = 0x16u8, I = 0x17u8, O = 0x18u8, P = 0x19u8,
    OpSqBk = 0x1Au8, ClSqBk = 0x1Bu8,
    Enter = 0x1Cu8, LCtrl = 0x1Du8,
    A = 0x1Eu8, S = 0x1Fu8, D = 0x20u8, F = 0x21u8, G = 0x22u8,
    H = 0x23u8, J = 0x24u8, K = 0x25u8, L = 0x26u8,
    Smcln = 0x27u8, Quote = 0x28u8,
    BkTk = 0x29u8,
    LShift = 0x2Au8, BSlash = 0x2Bu8,
    Z = 0x2Cu8, X = 0x2Du8, C = 0x2Eu8, V = 0x2Fu8, B = 0x30u8, N = 0x31u8, M = 0x32u8,
    Comma = 0x33u8, Period = 0x34u8, FSlash = 0x35u8, RShift = 0x36u8,
    KpStar = 0x37u8,
    LAlt = 0x38u8,
    Space = 0x39u8,
    CapsLk = 0x3Au8,
    F1 = 0x3Bu8, F2 = 0x3Cu8, F3 = 0x3Du8, F4 = 0x3Eu8, F5 = 0x3Fu8,
    F6 = 0x40u8, F7 = 0x41u8, F8 = 0x42u8, F9 = 0x43u8, F10 = 0x44u8,
    NumLk = 0x45u8, ScrlLk = 0x46u8,
    Kp7 = 0x47u8, Kp8 = 0x48u8, Kp9 = 0x49u8, KpMinus = 0x4Au8,
    Kp4 = 0x4Bu8, Kp5 = 0x4Cu8, Kp6 = 0x4Du8, KpPlus = 0x4Eu8,
    Kp1 = 0x4Fu8, Kp2 = 0x50u8, Kp3 = 0x51u8,
    Kp0 = 0x52u8, KpPeriod = 0x53u8,
    F11 = 0x57u8, F12 = 0x58u8,
}

pub struct Keyboard;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct KeyPress {
    keypress_data: AtomicU16,
}

impl KeyPress {
    pub const fn new(
        keycode: u16,
        extended: bool,
    ) -> Self {
        Self {
            keypress_data: AtomicU16::new(keycode | (extended as u16) << 8)
        }
    }

    pub const fn default() -> Self {
        Self { keypress_data: AtomicU16(Key::default() as u16) }
    }

    pub fn equals_key(&self, other: KeyPress) -> bool {
        self.keypress_data.load(Ordering::Relaxed) == other.keypress_data.load(Ordering::Relaxed)
    }

    fn get_keycode(&self) -> u8 { (self.keypress_data.load(Ordering::Relaxed) & 0xFF) as u8 }
    fn get_metadata(&self) -> u8 { (self.keypress_data.load(Ordering::Relaxed) >> 8 & 0xFF) as u8 }
}

impl Keyboard {

    pub fn update_keypress(scancode: u8) {
        unsafe {
            if scancode == EXTENDED_BYTE {
                IS_EXTENDED = true;
            } else {
                let is_release = (scancode & RELEASE_BYTE) >> 6 == 1;
                let keypress = KeyPress::new(scancode as u16, IS_EXTENDED);
                if !is_release {
                    if KEYPRESS_STACK_POINTER < KEYPRESS_STACK_LENGTH - 1 {    //cap to stack length - 1 for one byte of safety padding at the end
                        KEYPRESS_STACK[KEYPRESS_STACK_POINTER] = keypress;
                        KEYPRESS_STACK_POINTER += 1;
                    } else {
                        return (); //doesn't register if stack is full
                    }
                } else {
                    for i in (0..KEYPRESS_STACK_POINTER).rev() {
                        let kp = &mut KEYPRESS_STACK[i as usize];
                        if kp.get_keycode() == scancode {
                            ptr::copy(kp_ptr + 1, kp_ptr, (KEYPRESS_STACK_POINTER - i) as usize); 
                            // above, kp_ptr + 1 is safe because of the extra padding byte we added earlier
                            KEYPRESS_STACK_POINTER -= 1;
                            break;
                        }
                    }
                }
                move_into_input_driver_func(keypress);
                IS_EXTENDED = false;
            }
        }
    }

}

/*impl Keyboard {

    pub fn update_keypress(scancode: u8) {
        let cur_keypress = MOST_RECENT_KEYPRESS.load(Ordering::Relaxed);
        if (scancode > RELEASE_THRESHOLD) {
            MOST_RECENT_KEYPRESS.store(scancode, Ordering::Relaxed);
        } else {
            if (scancode - cur_keypress == RELEASE_THRESHOLD) {
                MOST_RECENT_KEYPRESS.store(0x00, Ordering::Relaxed);
            }
            // call input function scancode as key
            unsafe {
                crate::sys::console::INPUT_BUFFER.update_action(scancode - cur_keypress);
            }
        }
    }

}*/