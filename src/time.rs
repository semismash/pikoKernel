use core::arch::asm;

unsafe fn read_cmos(reg: u8) -> u8 {
    unsafe {
        asm!("out 0x70, al", in("al") reg | 0x80);
        let value: u8;
        asm!("in al, 0x71", out("al") value);
        value
    }
}

pub fn delay_seconds(seconds: u32) {
    unsafe {
        for _ in 0..seconds {
            let mut current_sec = read_cmos(0x00);
            
            while read_cmos(0x00) == current_sec {
                core::hint::spin_loop();
            }
        }
    }
}
