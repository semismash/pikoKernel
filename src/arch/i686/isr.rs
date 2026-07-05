use crate::drivers::display;
use core::arch;

const PIC_EOI: u8 = 0x20;
const KBD_PORT: u16 = 0x60;

pub struct InterruptHandler;

#[repr(C)]
pub struct InterruptStackFrame {
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
}

// interrupt handlers
impl InterruptHandler {

    #[unsafe(no_mangle)] pub extern "x86-interrupt" 
    fn handle_bp(frame: InterruptStackFrame) { // 0x03
        crate::sys::console::println!("-- BREAKPOINT --");
        crate::sys::console::write_and_flush!("EIP: {:#X}", frame.eip);
    }

    #[unsafe(no_mangle)] pub extern "x86-interrupt" 
    fn handle_dbf(frame: InterruptStackFrame, err_code: u32) -> ! { // 0x08
        let is_ext = (err_code & 0x1) == 1;
        let is_idt = (err_code >> 1 & 0x1) == 1;
        let is_ldt = (err_code >> 2 & 0x1) == 1;
        let is_gdt = !is_idt && !is_ldt;

        let sel_idx = (err_code >> 3 & 0x1FFF);

        panic!("!!! DOUBLE FAULT !!!\n
            EIP: {:#X}", 
            frame.eip);
    }

    #[unsafe(no_mangle)] pub extern "x86-interrupt" 
    fn handle_gpf(frame: InterruptStackFrame, err_code: u32) -> ! {    // 0x0D
        let is_ext = (err_code & 0x1) == 1;
        let is_idt = (err_code >> 1 & 0x1) == 1;
        let is_ldt = (err_code >> 2 & 0x1) == 1;
        let is_gdt = !is_idt && !is_ldt;

        let sel_idx = (err_code >> 3 & 0x1FFF);

        panic!("-- GENERAL PROTECTION FAULT --\n
            EIP: {:#X} | Error Code: {:#X}", 
            frame.eip, err_code);
    }

    #[unsafe(no_mangle)] pub extern "x86-interrupt" 
    fn handle_pit(frame: InterruptStackFrame) {    // 0x20
        unsafe {
            arch::asm!(
                "out dx, al",
                in("dx") crate::arch::i686::idt::PIC1_COMMAND,
                in("al") PIC_EOI,
                options(nomem, nostack, preserves_flags)
            );
        }
        crate::sys::time::SysTime::tick();
        crate::sys::console::evaluate_typematic();
    }

    #[unsafe(no_mangle)] pub extern "x86-interrupt" 
    fn handle_kbd(frame: InterruptStackFrame) {    //0x21
        unsafe {
            let mut scancode: u8;
            arch::asm!(
                "in al, dx",
                in("dx") KBD_PORT,
                out("al") scancode, 
                options(nomem, nostack, preserves_flags)
            );
            let kbd_ptr = &raw mut crate::arch::i686::kbd::KEYBOARD;
            (*kbd_ptr).try_update_keypress(scancode);
            arch::asm!(
                "out dx, al",
                in("dx") crate::arch::i686::idt::PIC1_COMMAND,
                in("al") PIC_EOI,
                options(nomem, nostack, preserves_flags)
            );
        }
    }

}