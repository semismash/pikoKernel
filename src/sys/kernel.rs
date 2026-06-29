use crate::arch::i686;
use crate::arch::i686::gdt::GDTPointer;
use crate::drivers::display::ForegroundColor as FGColor;
use crate::drivers::display::BackgroundColor as BGColor;
use crate::drivers::display;
use crate::drivers::display::*;
use crate::sys;
use crate::drivers::display::{VGAWriter, BUFFER_WIDTH, BUFFER_HEIGHT};
use crate::arch::i686::vga;
use crate::drivers::display::ScreenCharacter;
use crate::sys::time;

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

        //set up IDT
        i686::idt::PIC::remap_PIC();
        i686::idt::IDT::initialize();

        //enable text and cursor
        i686::vga::enable_cursor(14, 15);

        sys::console::write!("Value of CS is {:X}\n", cs);
        sys::console::write!("Value of DS is {:X}\n", ds);

        for i in 0..entry_count {
            let raw_entry: u64 = unsafe { *gdt_ptr.add(i as usize) };
            sys::console::write!("Entry {}: {:X}\n", i, raw_entry);
        }
        sys::console::write_and_flush!();

        sys::console::println!("Hey, what's up :D!");
        sys::console::println!("Hey, what's up :D! (but it's red)", FGColor::Red);
        sys::console::print!("I'm a ", FGColor::Magenta);
        sys::console::print!("Rustacean", FGColor::Yellow);
        sys::console::println!(", what's up?", FGColor::Magenta);
        

        //time::delay_seconds(2);
        //panic!("asfdfasdfasgdewgw");
        //local_buffer.clear_screen(frame);
    }

    loop {}
}
