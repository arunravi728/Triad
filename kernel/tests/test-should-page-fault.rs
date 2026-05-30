#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::interrupts::idt::{IdtIndex, InterruptDescriptorTable};
use kernel::interrupts::utils::generate_page_fault;
use kernel::{exit_qemu, serial_print, serial_println, QemuExitCode};
use lazy_static::lazy_static;

bootloader_api::entry_point!(test_main);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.add_interrupt_handler(
            IdtIndex::PageFaultInterruptIndex,
            kernel::handler_with_error_code!(page_fault_interrupt_handler),
        );

        idt
    };
}

extern "C" fn page_fault_interrupt_handler(
    stack_frame: &kernel::interrupts::ExceptionStackFrame,
    error_code: u64,
) -> ! {
    kernel::serial_println!(
        "\nEXCEPTION: PAGE FAULT with error code {:?}\n{:#?}",
        error_code,
        &*stack_frame
    );

    kernel::serial_println!("[ok]");
    exit_qemu(kernel::QemuExitCode::Success);

    kernel::hlt()
}

fn test_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kernel::serial_println!("Page Fault Test");

    kernel::interrupts::testonly_gdt_init();
    IDT.load();

    serial_print!("Triggering a page fault");
    generate_page_fault();

    serial_println!("[Test did not trigger page fault]");
    exit_qemu(QemuExitCode::Failed);
    kernel::hlt()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    kernel::hlt()
}
