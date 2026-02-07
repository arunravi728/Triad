#![no_std]
#![no_main]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use triad::interrupts::idt::{IdtIndex, InterruptDescriptorTable};

use triad::exit_qemu;

const DOUBLE_FAULT_IST_INDEX: u8 = 0;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.add_interrupt_handler(
            IdtIndex::DoubleFaultInterruptIndex,
            triad::handler_with_error_code!(double_fault_interrupt_handler),
        )
        .set_interrupt_stack_table_offset(DOUBLE_FAULT_IST_INDEX as u8);

        idt
    };
}

extern "C" fn double_fault_interrupt_handler(
    stack_frame: &triad::interrupts::ExceptionStackFrame,
    error_code: u64,
) -> ! {
    triad::serial_println!(
        "\nEXCEPTION: DOUBLE FAULT with error code {:?}\n{:#?}",
        error_code,
        &*stack_frame
    );

    triad::serial_println!("[ok]");
    exit_qemu(triad::QemuExitCode::Success);

    triad::hlt()
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    triad::serial_println!("Stack Overflow Test");

    triad::interrupts::testonly_gdt_init();
    IDT.load();

    generate_stack_overflow();

    panic!("Execution continues after a double fault due to stack overflow.");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    triad::test_panic_handler(info);
}

#[allow(unconditional_recursion)]
fn generate_stack_overflow() {
    generate_stack_overflow();
    // Prevent tail recursion optimizations.
    volatile::Volatile::new(0).read();
}
