#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::interrupts::idt::{IdtIndex, InterruptDescriptorTable};
use kernel::interrupts::utils::generate_divide_by_zero_interrupt;
use kernel::{exit_qemu, serial_print, serial_println, QemuExitCode};
use lazy_static::lazy_static;

bootloader_api::entry_point!(test_main);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.add_interrupt_handler(
            IdtIndex::DivideErrorInterruptIndex,
            kernel::handler!(divide_by_zero_interrupt_handler),
        );

        idt
    };
}

extern "C" fn divide_by_zero_interrupt_handler(
    stack_frame: &kernel::interrupts::ExceptionStackFrame,
) -> ! {
    kernel::serial_println!("\nEXCEPTION: DIVIDE BY ZERO ERROR\n{:#?}", &*stack_frame);

    kernel::serial_println!("[ok]");
    exit_qemu(kernel::QemuExitCode::Success);

    kernel::hlt()
}

fn test_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kernel::serial_println!("Divide By Zero Error Test");

    kernel::interrupts::testonly_gdt_init();
    IDT.load();
    generate_divide_bye_zero_error();

    serial_println!("[Test did not call divide by zero error handler]");
    exit_qemu(QemuExitCode::Failed);
    kernel::hlt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info);
}

#[allow(unconditional_panic)]
fn generate_divide_bye_zero_error() {
    serial_print!("Generating divide by zero error");
    generate_divide_by_zero_interrupt();
}
