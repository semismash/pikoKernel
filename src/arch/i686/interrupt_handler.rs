use crate::drivers::display;

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
        
    }

    #[unsafe(no_mangle)] pub extern "x86-interrupt" 
    fn handle_dbf(frame: InterruptStackFrame, err_code: u32) { // 0x08
        let is_ext = (err_code & 0x1) == 1;
        let is_idt = (err_code >> 1 & 0x1) == 1;
        let is_ldt = (err_code >> 1 & 0x1) == 1;
        let is_gdt = !is_idt && !is_ldt;

        let sel_idx = (err_code >> 3 & 0x1FFF);

    }

    #[unsafe(no_mangle)] pub extern "x86-interrupt" 
    fn handle_gpf(_frame: InterruptStackFrame, err_code: u32) -> ! {    // 00D
        let is_ext = (err_code & 0x1) == 1;
        let is_idt = (err_code >> 1 & 0x1) == 1;
        let is_ldt = (err_code >> 1 & 0x1) == 1;
        let is_gdt = !is_idt && !is_ldt;

        let sel_idx = (err_code >> 3 & 0x1FFF);

        //print error msg

        //panic
        panic!("Critical fault detected! INT: 0x0D");
    }

    #[unsafe(no_mangle)] pub extern "x86-interrupt" 
    fn handle_pit(frame: InterruptStackFrame) {    // 0x20

    }

    #[unsafe(no_mangle)] pub extern "x86-interrupt" 
    fn handle_kbd(frame: InterruptStackFrame) {    //0x21

    }

}
