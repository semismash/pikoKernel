use crate::arch::i686::isr;

const SYSTEM_FREQ: usize = 1000; // in Hz
const DIVISOR: u16 = { 
    let div = (isr::PIT_FREQ + (SYSTEM_FREQ / 2)) / SYSTEM_FREQ;
    if (div >= 65536) { 0 } else { div as u16 }
};

pub struct SysTime;