use crate::drivers::display;
use crate::drivers::display::{VGAWriter, BUFFER_WIDTH, BUFFER_HEIGHT};
use crate::arch::i686::vga;
use core::fmt::Write;

pub(crate) static mut OS_BUFFER: VGAWriter = VGAWriter::new(Some(vga::update_cursor));
pub(crate) static FRAME: display::FramePointer = display::FramePointer(
    vga::VGA_BUFFER_ADR as *mut [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT]
);

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