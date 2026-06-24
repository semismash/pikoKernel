use core::arch;
use crate::drivers::display::BUFFER_WIDTH;

pub const VGA_BUFFER_ADR: *mut u8 = 0xb8000 as *mut u8;
const PORT_INDEX: u16 = 0x3D4u16;
const PORT_DATA: u16 = 0x3D5u16;

//from OSdev wiki
pub fn enable_cursor(cursor_start: u8, cursor_end: u8) {

    //line cursor: arch_i686_enable_cursor(14, 15)
    //block cursor: arch_i686_enable_cursor(0, 15)
    //modify for interrupts
    unsafe {
        arch::asm!("out dx, al", in("dx") PORT_INDEX, in("al") 0x0Au8, 
            options(nomem, nostack, preserves_flags));
        arch::asm!(
            "in al, dx",
            "and al, {mask}",
            "or al, {start}",
            "out dx, al",
            in("dx") PORT_DATA, mask = in(reg_byte) 0xC0u8, start = in(reg_byte) cursor_start, out("al") _,
            options(nomem, nostack, preserves_flags),
        );
        arch::asm!("out dx, al", in("dx") PORT_INDEX, in("al") 0x0Bu8, 
            options(nomem, nostack, preserves_flags));
        arch::asm!(
            "in al, dx",
            "and al, {mask}",
            "or al, {end}",
            "out dx, al",
            in("dx") PORT_DATA, mask = in(reg_byte) 0xE0u8, end = in(reg_byte) cursor_end, out("al") _,
            options(nomem, nostack, preserves_flags),
        );
    }

}

pub fn disable_cursor() {
    unsafe {
        arch::asm!("out dx, al", in("dx") PORT_INDEX, in("al") 0x0Au8, 
            options(nomem, nostack, preserves_flags));
        arch::asm!("out dx, al", in("dx") PORT_DATA, in("al") 0x20u8, 
            options(nomem, nostack, preserves_flags));
    }
}

pub fn update_cursor(row: usize, col: usize) {
    let pos = (row * BUFFER_WIDTH + col) as u16;

    unsafe {
        arch::asm!("out dx, al", in("dx") PORT_INDEX, in("al") 0x0Fu8, 
            options(nomem, nostack, preserves_flags));
        arch::asm!("out dx, al", in("dx") PORT_DATA, in("al") (pos & 0xFF) as u8, 
            options(nomem, nostack, preserves_flags));
        arch::asm!("out dx, al", in("dx") PORT_INDEX, in("al") 0x0Eu8, 
            options(nomem, nostack, preserves_flags));
        arch::asm!("out dx, al", in("dx") PORT_DATA, in("al") ((pos >> 8) & 0xFF) as u8, 
            options(nomem, nostack, preserves_flags));
    }

}
