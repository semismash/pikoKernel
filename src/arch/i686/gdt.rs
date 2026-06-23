#[repr(C)]
struct GlobalDescriptor {
    null: GDTEntry,
    kernel_code: GDTEntry,
    kernel_data: GDTEntry,
    user_code: GDTEntry,
    user_data: GDTEntry,
}

struct SegmentType {
    A: bool,    // access
    RW: bool,   // read/write
    DC: bool,   // direction/conforming
    E: bool,    // executable
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum AccessLevel {
    KernelMode,
    Ring1Mode, // generally not used
    Ring2Mode, // generally not used
    UserMode,
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

#[repr(C, packed)]
struct GDTPointer {
    limit: u16,
    base: u32,
}

impl GlobalDescriptor {
    
    fn set_gdt () {

    }

    fn set_gate(
        &self,
        limit: u32,
        base: u32,
        seg_type: SegmentType,
        desc_type: bool,
        desc_access: AccessLevel,
        present: bool,

        granularity: bool,
    ) {

    }

}

macro_rules! encode_gate {
    ($seg:ident, $limit:expr, $base:expr, $access:expr, $gran:expr) => {
        {
            $seg.l_limit = $limit && 0xFFFF;
            $seg.l_base = $base && 0xFFFF;
            $seg.m_base = ($base >> 16) && 0xFF;
            $seg.access = $access;
            $seg.gran = ($limit >> 16) & 0x0F;
            $seg.gran |= $gran & 0xF0;
            $seg.h_base = (base >> 24) && 0xFF;
        }
    };
    ($($invalid:tt)*) => {
        compile_error!("Invalid use of encode_gate!");
    };
}
