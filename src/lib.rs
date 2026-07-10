#![feature(array_ptr_get)]
#![allow(warnings)]
#![no_std]
#![allow(non_snake_case)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(abi_x86_interrupt)]

#[macro_use]
pub mod sys;
#[macro_use]
pub mod drivers;
pub mod arch;
pub mod mem;
pub mod sub;
pub mod base;
pub mod utils;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    sys::kernel::main()
}
