#![no_std]
#![allow(non_snake_case)]

mod panichandler;
mod vga;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

//NOTE: THIS IS A TEST PROGRAM GENERATED WITH THE HELP OF AI. SUBJECT TO CHANGE SOON.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let message = b"Hello from pikoOS";
    let color: u8 = 0x0a;

    for (i, &byte) in message.iter().enumerate() {
        unsafe {
            *VGA_BUFFER.offset(i as isize * 2) = byte;
            *VGA_BUFFER.offset(i as isize * 2 + 1) = color;
        }
    }

    loop {}
}