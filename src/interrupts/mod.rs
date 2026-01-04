use lazy_static::lazy_static;

use crate::interrupts::idt::IdtIndex;

pub mod idt;
pub mod privilege;

mod interrupt_handlers;

lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.add_interrupt_handler(
            IdtIndex::DivideErrorIndex,
            interrupt_handlers::divide_error_handler,
        );

        idt
    };
}

pub fn init() {
    IDT.load();
}
