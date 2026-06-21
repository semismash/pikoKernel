#![no_std]
#![allow(non_snake_case)]
#![feature(ascii_char)]

mod panichandler;
mod display;
mod time; //debugging purpose

pub mod arch {
    pub mod i686 {
        pub mod vga;
    }
}

use display::*;
use display::ForegroundColor::*;
use arch::i686::vga;

const VGA_BUFFER_ADR: *mut u8 = 0xb8000 as *mut u8;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let message_1 = "Hey, what's up :D!";
    let message_2 = "\nHey, what's up :D! (but it's red)\n";
    let message_3 = "I'm a ";
    let message_4 = "Rustacean";
    let message_5 = ", what's up?\n";
    //let color: u8 = 0x0A;

    unsafe {
        vga::arch_i686_enable_cursor(14, 15);

        let mut local_buffer = display::VGABuffer::new(Some(vga::arch_i686_update_cursor));
        let vga_ref = 
            &mut *(VGA_BUFFER_ADR as *mut [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT]);

        local_buffer.clear(); //ALWAYS INITIALIZE THE BUFFER WITH A clear() BEFORE USING.
        local_buffer.write_plain_text_to_buf(message_1);
        local_buffer.write_fmt_text_to_buf(message_2, Red, None, None);
        local_buffer.write_fmt_text_to_buf(message_3, Magenta, None, None);
        local_buffer.write_fmt_text_to_buf(message_4, Yellow, None, None);
        local_buffer.write_fmt_text_to_buf(message_5, Magenta, None, None);
        
        local_buffer.flush(vga_ref);
        
    }

    loop {}
}