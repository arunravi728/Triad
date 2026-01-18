use lazy_static::lazy_static;
use x86_64::addr::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            // This needs to be a static mut. If this was immutable, the bootloader would make it
            // a read only page.
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

// The Interrupt Stack Table (IST) is part of the legacy Task State Segment (TSS).
//
// In 32 bit mode, the TSS was used for hardware context switching.
//
// In 64 bit mode, the TSS hold no task specific implementation. Instead it hold two stack tables -
// 1. The Interrupt Stack Table (IST)
// 2. The Privileged Stack Table (PST)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    reserved_1: u32,

    // The stack pointers used when a privilege level change occurs.
    pub privilege_stack_table: [VirtAddr; 3],
    reserved_2: u64,

    // The stack pointers used when an IDT entry has an IST value other than 0.
    pub interrupt_stack_table: [VirtAddr; 7],

    reserved_3: u64,
    reserved_4: u16,

    // The 16-bit offset to the I/O permission bit map from the 64-bit TSS base. It must not
    // exceed 0xDFFF.
    pub iomap_base: u16,
}

impl TaskStateSegment {
    #[inline]
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            privilege_stack_table: [VirtAddr::zero(); 3],
            interrupt_stack_table: [VirtAddr::zero(); 7],
            iomap_base: size_of::<TaskStateSegment>() as u16,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }
}

#[test_case]
fn test_check_tss_size() {
    // The minimum size of a TSS is 0x68 bytes.
    assert_eq!(size_of::<TaskStateSegment>(), 0x68);
}
