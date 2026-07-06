use crate::arch::i686::kbd::Key::P;
use crate::drivers::{BUFFER_CAPACITY, display};
use crate::drivers::display::{DisplayWriter, BUFFER_WIDTH, BUFFER_HEIGHT};
use crate::arch::i686::vga;
use crate::sub::spin::SpinLock;
use crate::sys::EchoMode::Immediate;
use crate::sys::kernel::OS_CONSOLE;
use core::fmt::Write;
use core::ascii::Char;
use crate::drivers::input;
use crate::drivers::input::{InputBuffer, InputAction};
use crate::arch::i686::kbd::{Keyboard, KeypressStack};

pub(crate) static FRAME: display::FramePointer = display::FramePointer(
    vga::VGA_BUFFER_ADR as *mut [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT]
);

pub(crate) static OS_BUFFER: SpinLock<DisplayWriter> = SpinLock::new(
    DisplayWriter::new(Some(vga::update_cursor)));
pub(crate) static INPUT_BUFFER: SpinLock<InputBuffer> = SpinLock::new(InputBuffer::new());
pub(crate) static TYPEMATIC: SpinLock<TypematicState> = SpinLock::new(TypematicState { cur_ticks: 0, is_repeating: false });

const TYPEMATIC_CHECK_DELAY_MS: u32 = 400;
const TYPEMATIC_SPAM_DELAY_MS: u32 = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EchoMode {
    None,
    Immediate,
    OnEnter,
    Silent,
    //Masked(Char)
}

pub struct TypematicState {
    pub cur_ticks: u32,
    pub is_repeating: bool,
}

impl TypematicState {
    pub fn reset(&mut self) {
        self.cur_ticks = 0;
        self.is_repeating = false;
    }
}

pub fn evaluate_typematic() {
    let mut tpm_lock = TYPEMATIC.lock();
    let console_lock = OS_CONSOLE.lock();

    let cur_action = console_lock.cur_action;
    if cur_action == InputAction::None {
        tpm_lock.reset();
        return;
    }

    tpm_lock.cur_ticks += 1;

    if !tpm_lock.is_repeating {
        if tpm_lock.cur_ticks >= TYPEMATIC_CHECK_DELAY_MS {
            tpm_lock.cur_ticks = 0;
            tpm_lock.is_repeating = true;
            //lock locally in the spam cycle to prevent corruption
            let mut input_buf = INPUT_BUFFER.lock();
            let mut os_buf = OS_BUFFER.lock();
            console_lock.execute_console_action(&mut *input_buf, &mut *os_buf);
        }
    } else {
        if tpm_lock.cur_ticks >= TYPEMATIC_SPAM_DELAY_MS {
            tpm_lock.cur_ticks = 0;
            let mut input_buf = INPUT_BUFFER.lock();
            let mut os_buf = OS_BUFFER.lock();
            console_lock.execute_console_action(&mut *input_buf, &mut *os_buf);
        }
    }
}

pub struct Console {
    pub cur_action: InputAction,
    echo_mode: EchoMode,
}

impl Console {

    pub const fn initialize() -> Self {
        Self {
            cur_action: InputAction::None,
            echo_mode: EchoMode::Immediate,
        }
    }

    pub fn set_echo_mode(&mut self, new_echo_mode: EchoMode) {
        self.echo_mode = new_echo_mode;
    }

    pub fn update_input(&mut self, kbd: &Keyboard, keypress_stack: &mut KeypressStack) {
        let mut input_buf = INPUT_BUFFER.lock();
        let mut os_buf = OS_BUFFER.lock();  
        if (self.echo_mode == EchoMode::None) { return; }
        let cur_stack_size = keypress_stack.stack_ptr;
        self.cur_action = input::get_action(&keypress_stack.stack, cur_stack_size);
        self.cur_action = input::apply_modifiers(self.cur_action, kbd);
        if self.cur_action == InputAction::Submit && DisplayWriter::get_row(os_buf.get_offset()) >= BUFFER_HEIGHT - 1 {
            self.cur_action = InputAction::None; 
            return;
        }
        self.execute_console_action(&mut *input_buf, &mut *os_buf);
    }

    pub fn execute_console_action(
        &self,
        input_buf: &mut InputBuffer,
        os_buf: &mut DisplayWriter,
    ) {
        unsafe {
            match self.echo_mode {
                EchoMode::None => {},
                EchoMode::Immediate => {
                    if !(matches!(self.cur_action, InputAction::AddChar(..)) || (self.cur_action == InputAction::Submit))
                        || !(os_buf.check_if_full() || input_buf.is_full())
                    {    
                        input_buf.execute_action(self.cur_action); 
                        os_buf.write_from_input_buf(&*input_buf);
                        os_buf.flush_sync(FRAME);
                    }
                },
                EchoMode::OnEnter => {  
                    if !(matches!(self.cur_action, InputAction::AddChar(..)) || self.cur_action == InputAction::Submit)
                        || !(os_buf.check_if_full() || input_buf.is_full()) 
                    {
                        input_buf.execute_action(self.cur_action);
                        os_buf.write_from_input_buf(&*input_buf);
                        if self.cur_action == InputAction::Submit { 
                            os_buf.flush_sync(FRAME); 
                        }
                    }
                },
                EchoMode::Silent => {
                    input_buf.execute_action(self.cur_action);
                },
                /*
                EchoMode::Masked(ch) => {
                
                }
                */
                _ => {}
            }
        }
    }

}

macro_rules! print {
    ($($args:tt)*) => {
        {
            let mut os_buf = $crate::sys::console::OS_BUFFER.lock();
            unsafe {
                $crate::drivers::display::print!(
                    &mut *os_buf,
                    $crate::sys::console::FRAME, 
                    $($args)*
                );
            }
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::print!");
    }; 
}
pub(crate) use print;

macro_rules! println {
    ($($args:tt)*) => {
        {
            let mut os_buf = $crate::sys::console::OS_BUFFER.lock();
            unsafe {
                $crate::drivers::display::println!(
                    &mut *os_buf, 
                    $crate::sys::console::FRAME, 
                    $($args)*
                );
            }
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::println!");
    };
}
pub(crate) use println;

macro_rules! write {
    ($($args:tt)*) => {
        {
            let mut os_buf = $crate::sys::console::OS_BUFFER.lock();
            unsafe {
                core::write!(
                    &mut *os_buf, 
                    $($args)*
                );
            }
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::write!");
    };
}
pub(crate) use write;

macro_rules! write_and_flush {
    () => {
        {
            let mut os_buf = $crate::sys::console::OS_BUFFER.lock();
            unsafe {
                os_buf.flush_sync($crate::sys::console::FRAME);
            }
        }
    };
    ($fmt:expr $(, $($args:tt)*)?) => {
        {
            let mut os_buf = $crate::sys::console::OS_BUFFER.lock();
            unsafe {
                $crate::drivers::display::write_and_flush!(
                    &mut *os_buf,
                    $crate::sys::console::FRAME,
                    $fmt
                    $(, $($args)*)?
                );
            }
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::write_and_flush!");
    };
}
pub(crate) use write_and_flush;
 
macro_rules! clear {
    ($($args:tt)*) => {
        {
            let mut os_buf = $crate::sys::console::OS_BUFFER.lock();
            unsafe {
                os_buf.clear_screen($crate::sys::console::FRAME);
            }
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::clear!");
    };
}
pub(crate) use clear;