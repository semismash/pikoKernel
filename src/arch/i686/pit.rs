use core::arch;

const PIT_DATA_0: u16 = 0x40u16;
const PIT_COMMAND: u16 = 0x43u16;

const SQUARE_WAVE_CMD: u8 = 0x36u8;

const PIT_FREQ: usize = 1_193_182; // Hz
const SYSTEM_FREQ: usize = 1000; // in Hz
const DIVISOR: u16 = { 
    let div = (PIT_FREQ + (SYSTEM_FREQ / 2)) / SYSTEM_FREQ;
    if (div >= 65536) { 0 } else { div as u16 }
};

pub struct PIT;

impl PIT {

    pub unsafe fn initialize() {
        let low_byte = (DIVISOR & 0xFF) as u8;
        let high_byte = ((DIVISOR >> 8) & 0xFF) as u8;
        core::arch::asm!(           //square wave command to command port
            "out dx, al",
            in("dx") PIT_COMMAND,
            in("al") 0x36u8,
            options(nomem, nostack, preserves_flags)
        );
        core::arch::asm!(           //send low byte from divisor to data port
            "out dx, al",
            in("dx") 0x40u16,
            in("al") low_byte,
            options(nomem, nostack, preserves_flags)
        );
        core::arch::asm!(           //send high byte from divisor to data port
            "out dx, al",
            in("dx") 0x40u16,
            in("al") high_byte,
            options(nomem, nostack, preserves_flags)
        );
    }

}