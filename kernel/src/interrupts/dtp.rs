use crate::memory::vaddr::VirtualAddress;

// A struct describing a pointer to a descriptor table (GDT / IDT).
// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    // Size of the DT.
    pub limit: u16,
    // Pointer to the memory region containing the DT.
    pub base: VirtualAddress,
}

#[test_case]
pub fn check_descriptor_pointer_size() {
    // Per the SDM, a descriptor pointer has to be 2+8=10 bytes
    assert_eq!(size_of::<DescriptorTablePointer>(), 10);
}
