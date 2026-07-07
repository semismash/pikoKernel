use core::arch;
use crate::arch::i686;
use crate::arch::i686::gdt::GDTPointer;
use crate::drivers::BackgroundColor::Green;
use crate::drivers::display::ForegroundColor as FGColor;
use crate::drivers::display::BackgroundColor as BGColor;
use crate::drivers::display;
use crate::drivers::display::*;
use crate::drivers::input::InputAction;
use crate::sub::spin::SpinLock;
use crate::sys;
use crate::drivers::display::{DisplayWriter, BUFFER_WIDTH, BUFFER_HEIGHT};
use crate::arch::i686::vga;
use crate::drivers::display::ScreenCharacter;
use crate::sys::time;
use crate::sys::Console;
use crate::drivers::display::{CURSOR_START, CURSOR_END};

use core::fmt::Write;

pub static OS_CONSOLE: SpinLock<Console> = SpinLock::new(Console::initialize());

pub fn main() -> ! {

    unsafe {
        //set up GDT
        let cs: u16;
        let ds: u16;
        let desc: GDTPointer;
        (cs, ds, desc) = i686::gdt::GDT::initialize();
        let entry_count = (desc.limit + 1) / 8;
        let gdt_ptr = desc.base as *const u64;

        //set up interrupts and I/O
        i686::idt::PIC::remap_PIC();
        i686::idt::IDT::initialize();
        i686::pit::PIT::initialize();

        //enable text and cursor
        i686::vga::enable_cursor(CURSOR_START, CURSOR_END);

        sys::console::println!("OS BOOT!");

        /*sys::console::write!("Value of CS is {:X}\n", cs);
        sys::console::write!("Value of DS is {:X}\n", ds);

        for i in 0..entry_count {
            let raw_entry: u64 = unsafe { *gdt_ptr.add(i as usize) };
            sys::console::write!("Entry {}: {:X}\n", i, raw_entry);
        }
        sys::console::write_and_flush!();

        //test delay
        crate::sys::time::SysTime::delay(2000);*/

        sys::console::println!("OS BOOTED!");
        sys::console::println!("Red colored text", FGColor::Red);

        /*sys::console::print!("I'm a ", FGColor::Magenta);
        sys::console::print!("Rustacean", FGColor::Yellow);
        sys::console::println!(", what's up?", FGColor::Magenta);*/

        /*for i in 0..50 {
            sys::time::SysTime::delay(100);
            sys::console::write_and_flush!("Number: {}\n", i)
        }*/

        //time::delay(2000);
        //panic!("asfdfasdfasgdewgw");
        //local_buffer.clear_screen(frame);
    }

    loop {}
}
