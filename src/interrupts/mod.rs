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
                // Save state of all mutable registers before interrupt handler call.
                // We need to do this to not get a page fault when the ISR returns.
                "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",

                // Calculate pointer to ExceptionStackFrame
                // The frame is pushed by the CPU before the pushes above.
                // 9 registers were pushed (9 * 8 bytes = 72).
                "mov rdi, rsp",
                "add rdi, 72",

                // Align stack to 16-bytes - x86_64 ABI requires 16-byte alignment before calls.
                "sub rsp, 8",
                "call {handler_fn}",
                "add rsp, 8",

                // Restore state before returning from the ISR. This will allow the return
                // address to properly populated and will prevent page faults on program resumption.
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rax",

                // Interrupt Return
                "iretq",
                handler_fn = sym $name,
            );
        }
        wrapper
    }}
}

macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[unsafe(naked)]
        extern "C" fn wrapper() -> ! {
            core::arch::naked_asm!(
                // Save state of all mutable registers before interrupt handler call.
                // We need to do this to not get a page fault when the ISR returns.
                "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",

                // Load error code into rsi
                "mov rsi, [rsp + 9*8]",

                // Calculate pointer to ExceptionStackFrame
                // The frame is pushed by the CPU before the pushes above.
                // 9 registers were pushed (9 * 8 bytes = 72)
                // 1 error code was pushed (1 * 8 bytes = 8)
                "mov rdi, rsp",
                "add rdi, 80",

                // Align stack to 16-bytes - x86_64 ABI requires 16-byte alignment before calls.
                "sub rsp, 8",
                "call {handler_fn}",
                "add rsp, 8",

                // Restore state before returning from the ISR. This will allow the return
                // address to properly populated and will prevent page faults on program resumption.
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rax",

                // Pop error code from stack
                "add rsp, 8",

                // Interrupt Return
                "iretq",
                handler_fn = sym $name,
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

        idt.add_interrupt_handler(
            IdtIndex::BreakpointInterruptIndex,
            handler!(breakpoint_interrupt_handler),
        );

        idt.add_interrupt_handler(
            IdtIndex::DoubleFaultInterruptIndex,
            handler_with_error_code!(double_fault_interrupt_handler),
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

extern "C" fn breakpoint_interrupt_handler(stack_frame: &ExceptionStackFrame) {
    crate::println!("\nEXCEPTION: BREAKPOINT\n{:#?}", &*stack_frame);
}

// The double fault error code is always 0. x86 expects the double fault handler to be diverging.
extern "C" fn double_fault_interrupt_handler(
    stack_frame: &ExceptionStackFrame,
    error_code: u64,
) -> ! {
    crate::println!(
        "\nEXCEPTION: DOUBLE FAULT with error code {:?}\n{:#?}",
        error_code,
        &*stack_frame
    );

    loop {}
}
