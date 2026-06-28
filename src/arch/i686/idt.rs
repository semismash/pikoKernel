use core::{arch, mem::MaybeUninit};

const PIC1_COMMAND: u16 = 0x20; // master PIC
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0; // slave PIC
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW3_PIC1_CASCADE: u8 = 0x04;
const ICW3_PIC2_CASCADE: u8 = 0x02;
const ICW4_8086: u8 = 0x01;

const PIC1_OFFSET: u8 = 0x20;  // offset of 32 to master
const PIC2_OFFSET: u8 = 0x28;  // offset of 40 to slave

const PIC_EOI: u8 = 0x20;

const IDT_ENTRY_COUNT: usize = 256;

static mut KERNEL_IDT: [IDTEntry; IDT_ENTRY_COUNT] = [IDTEntry::set_zero(); IDT_ENTRY_COUNT];
static mut IDT_DESCRIPTOR: IDTPointer = IDTPointer { limit: 0, base: 0 };

struct IDT;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IDTEntry {
    l_offset: u16,      // 0-15
    selector: u16,  // 16-31
    reserved: u8,       // 32-39; always set to 0
    attributes: u8,     // 40-47; gate type (40-43), storage segment (44, always 0), privelege level (45-46), present (47)
    h_offset: u16,      // 48-63
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IDTPointer {
    limit: u16,
    base: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum AccessLevel {
    KernelMode = 0,
    Ring1Mode = 1, // generally not used
    Ring2Mode = 2, // generally not used
    UserMode = 3,
}

struct PIC;

impl PIC {

    fn remap_PIC() {
        unsafe {
            let mut mask_1: u8;
            let mut mask_2: u8;

            //save masks
            arch::asm!("in al, dx", in("dx") PIC1_DATA, out("al") mask_1, options(nomem, nostack, preserves_flags));
            arch::asm!("in al, dx", in("dx") PIC2_DATA, out("al") mask_2, options(nomem, nostack, preserves_flags));

            //ICW1: initialization
            arch::asm!("out dx, al", in("dx") PIC1_COMMAND, in("al") ICW1_INIT, options(nomem, nostack, preserves_flags));
            PIC::io_delay();
            arch::asm!("out dx, al", in("dx") PIC2_COMMAND, in("al") ICW1_INIT, options(nomem, nostack, preserves_flags));
            PIC::io_delay();

            //ICW2: set vector offets
            arch::asm!("out dx, al", in("dx") PIC1_DATA, in("al") PIC1_OFFSET, options(nomem, nostack, preserves_flags));
            PIC::io_delay();
            arch::asm!("out dx, al", in("dx") PIC2_DATA, in("al") PIC2_OFFSET, options(nomem, nostack, preserves_flags));
            PIC::io_delay();

            //ICW3: configure cascading
            arch::asm!("out dx, al", in("dx") PIC1_DATA, in("al") ICW3_PIC1_CASCADE, options(nomem, nostack, preserves_flags));
            PIC::io_delay();
            arch::asm!("out dx, al", in("dx") PIC2_DATA, in("al") ICW3_PIC2_CASCADE, options(nomem, nostack, preserves_flags));
            PIC::io_delay();

            //ICW4: set environment mode to 8086
            arch::asm!("out dx, al", in("dx") PIC1_DATA, in("al") ICW4_8086, options(nomem, nostack, preserves_flags));
            PIC::io_delay();
            arch::asm!("out dx, al", in("dx") PIC2_DATA, in("al") ICW4_8086, options(nomem, nostack, preserves_flags));
            PIC::io_delay();

            //restore masked interrupts
            arch::asm!("out dx, al", in("dx") PIC1_DATA, in("al") mask_1, options(nomem, nostack, preserves_flags));
            arch::asm!("out dx, al", in("dx") PIC2_DATA, in("al") mask_2, options(nomem, nostack, preserves_flags));
        }
    }

    #[inline(always)]
    unsafe fn io_delay() {
        arch::asm!("out 0x80, al", in("al") 0u8, options(nomem, nostack, preserves_flags));
    }

}


impl IDT {

    pub unsafe fn initialize() {
        unsafe {
            // set up IDT and interrupt handlers

            IDT_DESCRIPTOR = IDTPointer {
                limit: (core::mem::size_of::<IDTEntry>() * IDT_ENTRY_COUNT - 1) as u16,
                base: &raw const KERNEL_IDT as *const IDTEntry as u32,
            };
            arch::asm!("lidt [{ptr}]", ptr = in(reg) &raw const IDT_DESCRIPTOR, options(nostack, preserves_flags));
        }
    }

}

impl IDTEntry {
    
    const fn set_zero() -> Self {
        Self {
            l_offset: 0,
            selector: 0,
            reserved: 0,
            attributes: 0,
            h_offset: 0,
        }
    }

    fn set_gate(
        isr_address: u32,
        selector: u16,
        gate_type: u8,
        access_lvl: AccessLevel,
        present: bool,
    ) -> Self {
        Self {
            l_offset: (isr_address & 0xFFFF) as u16,
            selector: selector,
            reserved: 0x00,
            attributes: (gate_type & 0xF) as u8 | (access_lvl as u8) << 5 | ((present as u8) << 7),
            h_offset: ((isr_address >> 16) & 0xFFFF) as u16,
        }
    }

    fn set_from_hex(val: u64) -> Self {
        Self {
            l_offset: (val & 0xFFFF) as u16,
            selector: ((val >> 16) & 0xFFFF) as u16,
            reserved: 0x00,
            attributes: ((val >> 40) & 0xFF) as u8,
            h_offset: ((val >> 48) & 0xFFFF) as u16,
        }
    }

}