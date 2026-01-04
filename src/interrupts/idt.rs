use bit_field::BitField;
use core::ops::RangeInclusive;

use crate::interrupts::privilege::KernelRings;

#[derive(Clone, Debug)]
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
        let mut options = IdtEntryOptions(0);
        options.set_present(true).set_gate_type(GateType::InterruptGateType);
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
    assert_eq!(options.present(), true);
    assert_eq!(options.gate_type(), GateType::InterruptGateType);
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
        .set_descriptor_privilege_level(KernelRings::Ring0)
        .set_gate_type(GateType::InterruptGateType)
        .set_interrupt_stack_table_offset(1);

    assert_eq!(options.present(), true);
    assert_eq!(options.descriptor_privilege_level(), KernelRings::Ring0);
    assert_eq!(options.gate_type(), GateType::InterruptGateType);
    assert_eq!(options.interrupt_stack_table_offset(), 1);
}
