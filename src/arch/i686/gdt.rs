use core::arch;

use crate::arch::i686::vga;

static mut KERNEL_GDT: GDT = GDT::new_empty();
static mut GDT_DESCRIPTOR: GDTPointer = GDTPointer { limit: 0 , base: 0 };

#[repr(C, align(8))]
pub struct GDT {
    null: GDTEntry,
    kernel_code: GDTEntry,
    kernel_data: GDTEntry,
    user_code: GDTEntry,
    user_data: GDTEntry,
}

#[repr(C, packed)]
struct GDTPointer {
    limit: u16,
    base: u32,
}

#[derive(Debug, Clone, Copy)]
struct SegmentType {
    A: bool,    // access (40)
    RW: bool,   // read/write (41),
    DC: bool,   // direction/conforming (42)
    E: bool,    // executable (43)
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum AccessLevel {
    KernelMode = 0,
    Ring1Mode = 1, // generally not used
    Ring2Mode = 2, // generally not used
    UserMode = 3,
}

#[repr(C, packed)]
struct GDTEntry {
    l_limit: u16,
    l_base: u16,
    m_base: u8,
    access: u8,
    gran: u8,
    h_base: u8,
}

impl GDT {

    const fn new_empty() -> Self {
        Self {
            null: GDTEntry::new_zero(),
            kernel_code: GDTEntry::new_zero(),
            kernel_data: GDTEntry::new_zero(),
            user_code: GDTEntry::new_zero(),
            user_data: GDTEntry::new_zero(),
        }
    }
    
    pub unsafe fn initialize() {
        unsafe {
            KERNEL_GDT = GDT {
                null: GDTEntry::set_from_hex(0x0000000000000000),
                kernel_code: GDTEntry::set_gate(     // 0x00CF9A000000FFFF
                    0xFFFFF,
                    0x00000000,
                    SegmentType { A: false, RW: true, DC: false, E: true },
                    true,
                    AccessLevel::KernelMode,
                    true,
                    true,
                    true,
                ),
                kernel_data: GDTEntry::set_gate(    // 0x00CF92000000FFFF
                    0xFFFFF,
                    0x00000000,
                    SegmentType { A: false, RW: true, DC: false, E: false },
                    true,
                    AccessLevel::KernelMode,
                    true,
                    true,
                    true,
                ),
                user_code: GDTEntry::set_gate(      // 0x00CFFA000000FFFF
                    0xFFFFF,
                    0x00000000,
                    SegmentType { A: false, RW: true, DC: false, E: true },
                    true,
                    AccessLevel::UserMode,
                    true,
                    true,
                    true,
                ),
                user_data: GDTEntry::set_gate(      // 0x00CFF2000000FFFF
                    0xFFFFF,
                    0x00000000,
                    SegmentType { A: false, RW: true, DC: false, E: false },
                    true,
                    AccessLevel::UserMode,
                    true,
                    true,
                    true,
                ),
            };
            GDT_DESCRIPTOR = GDTPointer {
                limit: (core::mem::size_of::<GDT>() - 1) as u16,
                base: &raw const KERNEL_GDT as *const GDT as u32,
            };

            GDT::flush(&raw const GDT_DESCRIPTOR);

            // check values of code and data segment registers (should be 0x08 and 0x10 respectively)
            /*let cs: u16;
            let ds: u16;
            unsafe {
                core::arch::asm!(
                    "mov {0:x}, cs",
                    "mov {1:x}, ds",
                    out(reg) cs,
                    out(reg) ds,
                );
            }

            crate::drivers::display::println!();*/

        }
    }

    unsafe fn flush(gdt_ptr: *const GDTPointer) {
        unsafe {
            arch::asm!(
                "lgdt [{ptr}]",
                "push 0x08",
                "lea {tmp}, [2f]",
                "push {tmp}",
                "retf",   
                "2:",
                "mov {tmp}, 0x10",
                "mov ds, {tmp}",
                "mov es, {tmp}",
                "mov fs, {tmp}",
                "mov gs, {tmp}",
                "mov ss, {tmp}",
                ptr = in(reg) gdt_ptr,
                tmp = out(reg) _,
                options(nostack, preserves_flags)
            );
        }
    }

}

impl GDTEntry {

    const fn new_zero() -> Self {
        Self {
            l_limit: 0,
            l_base: 0,
            m_base: 0,
            access: 0,
            gran: 0,
            h_base: 0,
        }
    }

    fn set_gate(    //to be used later
        limit: u32,                 // 0-15, 48-51
        base: u32,                  // 16-31, 32-39, 56-63
        seg_type: SegmentType,      // 40-43
        desc_type: bool,            // 44
        desc_access: AccessLevel,   // 45-46
        present: bool,              // 47
        default_size: bool,         // 54
        granularity: bool,          // 55
        //bit 52 (AVL) remains unused, bit 53 (A) is always 0 on 32-bit
    ) -> Self {
        Self {
            l_limit: (limit & 0xFFFF) as u16,
            l_base: (base & 0xFFFF) as u16,
            m_base: ((base >> 16) & 0xFF) as u8,
            access: 
                (seg_type.A as u8 | (seg_type.RW as u8) << 1 | (seg_type.DC as u8) << 2 | (seg_type.E as u8) << 3) | 
                ((desc_type as u8) << 4 | (desc_access as u8 & 0x3) << 5 | (present as u8) << 7),
            gran: ((limit >> 16) & 0xF) as u8 | (default_size as u8) << 6 | (granularity as u8) << 7,
            h_base: ((base >> 24) & 0xFF) as u8,
        }
    }
    
    const fn set_from_hex(val: u64) -> Self {
        Self {
            l_limit: (val & 0xFFFF) as u16,
            l_base: ((val >> 16) & 0xFFFF) as u16,
            m_base: ((val >> 32) & 0xFF) as u8,
            access: ((val >> 40) & 0xFF) as u8,
            gran: ((val >> 48) & 0xFF) as u8,
            h_base: ((val >> 56) & 0xFF) as u8,
        }
    }

}