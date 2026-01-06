use lazy_static::lazy_static;

use crate::interrupts::idt::IdtIndex;

pub mod idt;
pub mod privilege;

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

// The function prologue is a few lines of code at the beginning of a function, which prepare the
// stack and registers for use within the function. Thus, the prologue generally tends to write
// information before we can access the interrupt stack frame. To access the correct addresses,
// we need to run functions without a prologue. The [naked] attribute helps with this. Note we
// cannot call anything but a naked_asm! call from a naked function. Hence, link Rust functions
// to handle specific exceptions.
macro_rules! handler {
    ($name: ident) => {{
        #[unsafe(naked)]
        extern "C" fn wrapper() -> ! {
            core::arch::naked_asm!(
                "mov rdi, rsp",
                "sub rsp, 8", // Required to align the stack
                "call {}",
                sym $name,
            );
        }
        wrapper
    }}
}

lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.add_interrupt_handler(
            IdtIndex::DivideErrorInterruptIndex,
            handler!(divide_error_handler),
        );
        idt.add_interrupt_handler(
            IdtIndex::InvalidOpcodeInterruptIndex,
            handler!(invalid_opcode_handler),
        );

        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "C" fn divide_error_handler(stack_frame: &ExceptionStackFrame) -> ! {
    crate::println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", &*stack_frame);
    loop {}
}

extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) -> ! {
    crate::println!("\nEXCEPTION: INVALID OPCODE\n{:#?}", &*stack_frame);
    loop {}
}
