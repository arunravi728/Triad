use bit_field::BitField;
use core::ops::{Index, IndexMut, RangeInclusive};

use crate::interrupts::privilege::KernelRings;

// This is the interrupt handler type for the IDT. It needs to be a function type with a defined
// calling convention, as it is directly called by hardware (a calling convention is an
// low-level implementation-level scheme for how subroutines or functions receive parameters from
// their caller and how they return a result). The calling convention used here is the C standard,
// which is a standard in OS development. This function is never called, rather the hardware jumps
// to it. A by-product of this is that the function will never return (a diverging function).
pub type InterruptHandler = extern "C" fn() -> !;

// The various interrupt handlers on x86 machines can be found here -
// https://wiki.osdev.org/Interrupt_Descriptor_Table#IDT_items
#[derive(Clone, Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub divide_error_interrupt: IdtEntry,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum IdtIndex {
    DivideErrorIndex = 0,
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        InterruptDescriptorTable {
            divide_error_interrupt: IdtEntry::empty(),
        }
    }

    pub fn add_interrupt_handler(
        &mut self,
        interrupt_index: IdtIndex,
        handler: InterruptHandler,
    ) -> &mut IdtEntryOptions {
        // TODO: Pass in the CS segment.
        self[interrupt_index as u8] = IdtEntry::new(handler, 0);
        &mut self[interrupt_index as u8].idt_entry_options
    }
}

impl Index<u8> for InterruptDescriptorTable {
    type Output = IdtEntry;

    #[inline]
    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.divide_error_interrupt,
            i => panic!("Unsupported interrupt index {i}"),
        }
    }
}

impl IndexMut<u8> for InterruptDescriptorTable {
    #[inline]
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.divide_error_interrupt,
            i => panic!("Unsupported interrupt index {i}"),
        }
    }
}

// The layout of the IdtEntry can be found at -
// https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64
#[derive(Debug, Clone, Copy)]
#[repr(C)]
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

impl IdtEntry {
    // TODO: Implement the SegmentSelector structure when implementing the GDT.
    fn new(handler: InterruptHandler, segement_selector: u16) -> Self {
        // The address to the handler is a 64 bit value.
        let isr_address = handler as u64;
        IdtEntry {
            isr_address_low: isr_address as u16,
            isr_address_mid: (isr_address >> 16) as u16,
            isr_address_high: (isr_address >> 32) as u32,
            segment_selector: segement_selector,
            // Disables interrupts and marks the descriptor as valid.
            idt_entry_options: *IdtEntryOptions::new()
                .set_present(true)
                .set_gate_type(GateType::InterruptGateType),
            reserved: 0,
        }
    }

    fn empty() -> Self {
        IdtEntry {
            isr_address_low: 0,
            isr_address_mid: 0,
            isr_address_high: 0,
            segment_selector: 0,
            idt_entry_options: IdtEntryOptions::new(),
            reserved: 0,
        }
    }
}

// The layout of the IdtEntryOptions can be found at -
// https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64
//
// Bits 0 - 2: Interrupt Stack Table Offset
// Bits 3 - 7: Reserved
// Bits 8 - 11: Gate Type - Can be Interrupt Gate (0xE) or Trap Gate (0xF)
// Bit 12 - Must be 0
// Bits 13 - 14: Descriptor Privilege Level
// Bit 15: Present - Must be set (1) for the descriptor to be valid.
#[derive(Debug, Clone, Copy)]
pub struct IdtEntryOptions(u16);

impl IdtEntryOptions {
    const INTERRUPT_STACK_TABLE_OFFSET_BITS: RangeInclusive<usize> = 0..=2;
    const GATE_TYPE_BITS: RangeInclusive<usize> = 8..=11;
    const DESCRIPTOR_PRIVILEGE_BITS: RangeInclusive<usize> = 13..=14;
    const PRESENT_BIT: usize = 15;

    fn new() -> Self {
        let options = IdtEntryOptions(0);
        options
    }

    fn value(&self) -> u16 {
        self.0
    }

    fn mut_value(&mut self) -> &mut u16 {
        &mut self.0
    }

    #[cfg(test)]
    fn present(&self) -> bool {
        self.value().get_bit(IdtEntryOptions::PRESENT_BIT)
    }

    fn set_present(&mut self, present: bool) -> &mut Self {
        self.mut_value()
            .set_bit(IdtEntryOptions::PRESENT_BIT, present);
        self
    }

    #[cfg(test)]
    fn descriptor_privilege_level(&self) -> KernelRings {
        KernelRings::new(
            self.value()
                .get_bits(IdtEntryOptions::DESCRIPTOR_PRIVILEGE_BITS),
        )
    }

    fn set_descriptor_privilege_level(&mut self, kernel_ring: KernelRings) -> &mut Self {
        self.mut_value().set_bits(
            IdtEntryOptions::DESCRIPTOR_PRIVILEGE_BITS,
            kernel_ring as u16,
        );
        self
    }

    #[cfg(test)]
    fn gate_type(&self) -> GateType {
        GateType::new(self.value().get_bits(IdtEntryOptions::GATE_TYPE_BITS))
    }

    fn set_gate_type(&mut self, gate_type: GateType) -> &mut Self {
        self.mut_value()
            .set_bits(IdtEntryOptions::GATE_TYPE_BITS, gate_type as u16);
        self
    }

    #[cfg(test)]
    fn interrupt_stack_table_offset(&self) -> u16 {
        self.value()
            .get_bits(IdtEntryOptions::INTERRUPT_STACK_TABLE_OFFSET_BITS)
    }

    fn set_interrupt_stack_table_offset(&mut self, offset: u8) -> &mut Self {
        if offset > 7 {
            panic!("Interrupt stack table offset is a bit value, cannot be greater than 7.")
        }

        self.mut_value().set_bits(
            IdtEntryOptions::INTERRUPT_STACK_TABLE_OFFSET_BITS,
            offset as u16,
        );
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GateType {
    InterruptGateType = 0x0E,
    TrapGateType = 0x0F,
}

impl GateType {
    pub fn new(gate_type: u16) -> Self {
        match gate_type {
            0x0E => GateType::InterruptGateType,
            0x0F => GateType::TrapGateType,
            _ => {
                panic!("Illegal Gate Type")
            }
        }
    }
}

#[test_case]
fn test_idt_entry_options_construction() {
    let options: IdtEntryOptions = IdtEntryOptions::new();
    assert_eq!(options.present(), false);
}

#[test_case]
fn test_idt_entry_options_privilege_level() {
    let mut options: IdtEntryOptions = IdtEntryOptions::new();

    options.set_descriptor_privilege_level(KernelRings::Ring0);
    assert_eq!(options.descriptor_privilege_level(), KernelRings::Ring0);

    options.set_descriptor_privilege_level(KernelRings::Ring1);
    assert_eq!(options.descriptor_privilege_level(), KernelRings::Ring1);

    options.set_descriptor_privilege_level(KernelRings::Ring2);
    assert_eq!(options.descriptor_privilege_level(), KernelRings::Ring2);

    options.set_descriptor_privilege_level(KernelRings::Ring3);
    assert_eq!(options.descriptor_privilege_level(), KernelRings::Ring3);
}

#[test_case]
fn test_idt_entry_options_gate_type() {
    let mut options: IdtEntryOptions = IdtEntryOptions::new();

    options.set_gate_type(GateType::InterruptGateType);
    assert_eq!(options.gate_type(), GateType::InterruptGateType);

    options.set_gate_type(GateType::TrapGateType);
    assert_eq!(options.gate_type(), GateType::TrapGateType);
}

#[test_case]
fn test_idt_entry_options_ist_offset() {
    let mut options: IdtEntryOptions = IdtEntryOptions::new();

    options.set_interrupt_stack_table_offset(0);
    assert_eq!(options.interrupt_stack_table_offset(), 0);

    options.set_interrupt_stack_table_offset(1);
    assert_eq!(options.interrupt_stack_table_offset(), 1);

    options.set_interrupt_stack_table_offset(7);
    assert_eq!(options.interrupt_stack_table_offset(), 7);
}

#[test_case]
fn test_idt_entry_options_chained_mutation() {
    let mut options: IdtEntryOptions = IdtEntryOptions::new();
    options
        .set_present(true)
        .set_descriptor_privilege_level(KernelRings::Ring0)
        .set_gate_type(GateType::InterruptGateType)
        .set_interrupt_stack_table_offset(1);

    assert_eq!(options.present(), true);
    assert_eq!(options.descriptor_privilege_level(), KernelRings::Ring0);
    assert_eq!(options.gate_type(), GateType::InterruptGateType);
    assert_eq!(options.interrupt_stack_table_offset(), 1);
}

#[test_case]
fn test_idt_entry_construction() {
    extern "C" fn test_handler() -> ! {
        crate::println!("TEST INTERRUPT HANDLER");
        loop {}
    }

    let test_handler_address = (test_handler as extern "C" fn() -> !) as u64;

    let idt_entry = IdtEntry::new(test_handler, /*segment_selector*/ 42);
    assert_eq!(idt_entry.isr_address_low, test_handler_address as u16);
    assert_eq!(
        idt_entry.isr_address_mid,
        (test_handler_address >> 16) as u16
    );
    assert_eq!(
        idt_entry.isr_address_high,
        (test_handler_address >> 32) as u32
    );
    assert_eq!(idt_entry.idt_entry_options.present(), true);
    assert_eq!(
        idt_entry.idt_entry_options.gate_type(),
        GateType::InterruptGateType
    );
}

#[test_case]
fn test_idt_divide_error_setup() {
    extern "C" fn divide_error_handler() -> ! {
        crate::println!("DIVIDE ERROR INTERRUPT HANDLER");
        loop {}
    }

    let mut idt = InterruptDescriptorTable::new();

    let divide_error_entry_options =
        idt.add_interrupt_handler(IdtIndex::DivideErrorIndex, divide_error_handler);

    assert_eq!(divide_error_entry_options.present(), true);
    assert_eq!(
        divide_error_entry_options.gate_type(),
        GateType::InterruptGateType
    );

    divide_error_entry_options.set_descriptor_privilege_level(KernelRings::Ring0);
    assert_eq!(
        divide_error_entry_options.descriptor_privilege_level(),
        KernelRings::Ring0
    );

    let divide_error_handler_address = (divide_error_handler as extern "C" fn() -> !) as u64;

    assert_eq!(
        idt[IdtIndex::DivideErrorIndex as u8].isr_address_low,
        divide_error_handler_address as u16
    );
    assert_eq!(
        idt[IdtIndex::DivideErrorIndex as u8].isr_address_mid,
        (divide_error_handler_address >> 16) as u16
    );
    assert_eq!(
        idt[IdtIndex::DivideErrorIndex as u8].isr_address_high,
        (divide_error_handler_address >> 32) as u32
    );
}
