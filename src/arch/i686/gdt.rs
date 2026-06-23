use core::arch;

#[repr(C, packed)]
struct GlobalDescriptor {
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

impl GlobalDescriptor {
    
    fn initialize () {

    }

    #[unsafe(naked)]
    fn flush() {
        unsafe {
            arch::naked_asm!(
                
            );
        }
    }

}

impl GDTEntry {

    fn set_gate(
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
    
    fn set_from_hex(&mut self, val: u64) -> Self {
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
