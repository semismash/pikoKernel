#![no_std]
#![no_main]

mod panichandler;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop {}
}