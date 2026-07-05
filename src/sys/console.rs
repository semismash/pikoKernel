use crate::drivers::display;
use crate::drivers::display::{DisplayWriter, BUFFER_WIDTH, BUFFER_HEIGHT};
use crate::arch::i686::vga;
use crate::sys::EchoMode::Immediate;
use core::fmt::Write;
use core::ascii::Char;
use crate::drivers::input;
use crate::drivers::input::{InputBuffer, InputAction};

pub(crate) static mut OS_BUFFER: DisplayWriter = DisplayWriter::new(Some(vga::update_cursor));
pub(crate) static FRAME: display::FramePointer = display::FramePointer(
    vga::VGA_BUFFER_ADR as *mut [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT]
);

pub(crate) static mut INPUT_BUFFER: InputBuffer = InputBuffer::new();

pub enum EchoMode {
    None,
    Immediate,
    OnEnter,
    Silent,
    //Masked(Char)
}

pub struct Console {
    cur_action: InputAction,
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

    pub fn update_input(&mut self) {
        let kbd_ptr = &raw const crate::arch::i686::kbd::KEYPRESS_STACK;
        let input_ptr = &raw mut INPUT_BUFFER;
        let os_ptr = &raw mut OS_BUFFER;
        unsafe { 
            let cur_stack_size = *(&raw const crate::arch::i686::kbd::KEYPRESS_STACK_POINTER);
            match self.echo_mode {
                EchoMode::None => {},
                EchoMode::Immediate => {
                    self.cur_action = input::get_action(&*kbd_ptr, cur_stack_size); 
                    if self.cur_action == InputAction::Submit && (*os_ptr).row_pos >= BUFFER_HEIGHT - 1 {
                        self.cur_action = InputAction::None;
                    }
                    if !(matches!(self.cur_action, InputAction::AddChar(..)) 
                        || (self.cur_action == InputAction::Submit))
                        || !((*os_ptr).check_if_full() || (*input_ptr).is_full())
                    {    
                        (*input_ptr).execute_action(self.cur_action); 
                        (*os_ptr).write_from_input_buf(&*input_ptr);
                        (*os_ptr).flush_sync(FRAME);
                    }
                },
                EchoMode::OnEnter => {  
                    self.cur_action = input::get_action(&*kbd_ptr, cur_stack_size);

                    if self.cur_action == InputAction::Submit && (*os_ptr).row_pos >= BUFFER_HEIGHT - 1 {
                        self.cur_action = InputAction::None;
                    }
                    if !(matches!(self.cur_action, InputAction::AddChar(..)) || self.cur_action == InputAction::Submit)
                        || !((*os_ptr).check_if_full() || (*input_ptr).is_full()) 
                    {
                        (*input_ptr).execute_action(self.cur_action);
                        (*os_ptr).write_from_input_buf(&*input_ptr);
                        if self.cur_action == InputAction::Submit { 
                            (*os_ptr).flush_sync(FRAME); 
                        }
                    }
                },
                EchoMode::Silent => {
                    self.cur_action = input::get_action(&*kbd_ptr, cur_stack_size);
                    (*input_ptr).execute_action(self.cur_action);
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
        unsafe {
            $crate::drivers::display::print!(
                &mut *&raw mut $crate::sys::console::OS_BUFFER, 
                $crate::sys::console::FRAME, 
                $($args)*
            );
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::print!");
    }; 
}
pub(crate) use print;

macro_rules! println {
    ($($args:tt)*) => {
        unsafe {
            $crate::drivers::display::println!(
                &mut *&raw mut $crate::sys::console::OS_BUFFER, 
                $crate::sys::console::FRAME, 
                $($args)*
            );
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::println!");
    };
}
pub(crate) use println;

macro_rules! write {
    ($($args:tt)*) => {
        unsafe {
            core::write!(
                &mut *&raw mut $crate::sys::console::OS_BUFFER, 
                $($args)*
            );
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::write!");
    };
}
pub(crate) use write;

macro_rules! write_and_flush {
    () => {
        unsafe {
            (*&raw mut $crate::sys::console::OS_BUFFER).flush_sync($crate::sys::console::FRAME);
        }
    };
    ($fmt:expr $(, $($args:tt)*)?) => {
        unsafe {
            $crate::drivers::display::write_and_flush!(
                &mut *&raw mut $crate::sys::console::OS_BUFFER,
                $crate::sys::console::FRAME,
                $fmt
                $(, $($args)*)?
            );
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::write_and_flush!");
    };
}
pub(crate) use write_and_flush;
 
macro_rules! clear {
    ($($args:tt)*) => {
        unsafe {
            (*&raw mut $crate::sys::console::OS_BUFFER).clear_screen($crate::sys::console::FRAME);
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid arguments passed to crate::sys::console::clear!");
    };
}
pub(crate) use clear;