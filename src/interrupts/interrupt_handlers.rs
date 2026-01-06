// This file contains the various interrupt handlers used by the IDT

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

pub(crate) extern "C" fn divide_error_handler() -> ! {
    let exception_stack_frame: &ExceptionStackFrame;

    let stack_ptr: usize;

    // We use inline assembly to retrieve the values of the rsp register which stores the interrupt
    // stack frame and store it in an ExceptionStackFrame instance.
    unsafe {
        core::arch::asm!(
            "mov {}, rsp",
        out(reg) stack_ptr,
        options(nomem, nostack, preserves_flags)
        );

        exception_stack_frame = &*(stack_ptr as *const ExceptionStackFrame);
    }

    crate::println!("DIVIDE ERROR INTERRUPT! {exception_stack_frame:#?}");
    loop {}
}
