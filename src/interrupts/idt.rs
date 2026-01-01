#[derive(Clone, Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {}

// The layout of the IdtEntry can be found at -
// https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    // This is a 64 bit address offset split into three 16-bit chunks. It represents the address of
    // the entry point of the Interrupt Service Routine.
    isr_address_low: u16,

    // TODO: Implement the SegmentSelector structure when implementing the GDT.
    segment_selector: u16,

    idt_entry_options: IdtEntryOptions,

    isr_address_mid: u16,
    isr_address_high: u32,

    reserved: u32,
}

// The layout of the IdtEntryOptions can be found at -
// https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntryOptions {}