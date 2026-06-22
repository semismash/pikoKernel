#![no_std]
#![allow(non_snake_case)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]

mod panichandler;
mod display;
mod time; //debugging purpose

pub mod arch {
    pub mod i686 {
        pub mod vga;
    }
}

use display::*;
use display::ForegroundColor as FGColor;
use display::BackgroundColor as BGColor;
use arch::i686::vga;

const VGA_BUFFER_ADR: *mut u8 = 0xb8000 as *mut u8;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let message_1 = "Hey, what's up :D!";
    let message_2 = "Hey, what's up :D! (but it's red)";
    let message_3 = "I'm a ";
    let message_4 = "Rustacean";
    let message_5 = ", what's up?";

    unsafe {
        vga::arch_i686_enable_cursor(14, 15);

        let mut local_buffer = display::VGABuffer::new(Some(vga::arch_i686_update_cursor));
        let vga_ref = 
            &mut *(VGA_BUFFER_ADR as *mut [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT]);

        local_buffer.clear(); //ALWAYS INITIALIZE THE BUFFER WITH A clear() BEFORE USING.
        
        println!(local_buffer, vga_ref, message_1);
        println!(local_buffer, vga_ref, message_2, FGColor::Red);
        print!(local_buffer, vga_ref, message_3, FGColor::Magenta);
        print!(local_buffer, vga_ref, message_4, FGColor::Yellow);
        println!(local_buffer, vga_ref, message_5, FGColor::Magenta);

        time::delay_seconds(2);
        local_buffer.clear_screen(vga_ref);
    }

    loop {}
}