use crate::arch::i686::gdt::GDTPointer;
use crate::drivers::display::*;
use crate::drivers::display::ForegroundColor as FGColor;
use crate::drivers::display::BackgroundColor as BGColor;
use crate::arch::i686;
use crate::time;

use core::fmt::Write;

pub fn main() -> ! {
    
    let message_1 = "Hey, what's up :D!";
    let message_2 = "Hey, what's up :D! (but it's red)";
    let message_3 = "I'm a ";
    let message_4 = "Rustacean";
    let message_5 = ", what's up?";

    unsafe {
        //set up GDT
        let cs: u16;
        let ds: u16;
        let desc: GDTPointer;
        (cs, ds, desc) = i686::gdt::GDT::initialize();
        let entry_count = (desc.limit + 1) / 8;
        let gdt_ptr = desc.base as *const u64;

        //enable text and cursor
        i686::vga::enable_cursor(14, 15);

        let mut local_buffer = VGABuffer::new(Some(i686::vga::update_cursor));
        let frame = 
            &mut *(i686::vga::VGA_BUFFER_ADR as *mut [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT]);

        write!(local_buffer, "Value of CS is {:X}\n", cs);
        write!(local_buffer, "Value of DS is {:X}\n", ds);

        for i in 0..entry_count {
            let raw_entry: u64 = unsafe { *gdt_ptr.add(i as usize) };
            write!(local_buffer, "Entry {}: {:X}\n", i, raw_entry);
        }
        local_buffer.flush(frame);
        
        println!(local_buffer, frame, message_1);
        println!(local_buffer, frame, message_2, FGColor::Red);
        print!(local_buffer, frame, message_3, FGColor::Magenta);
        print!(local_buffer, frame, message_4, FGColor::Yellow);
        println!(local_buffer, frame, message_5, FGColor::Magenta);

        /*time::delay_seconds(2);
        local_buffer.clear_screen(frame);*/
    }

    loop {}
}
