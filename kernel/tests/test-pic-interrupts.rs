#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::interrupts::idt::{IdtIndex, InterruptDescriptorTable};
use kernel::interrupts::pic::Pics;
use kernel::interrupts::ExceptionStackFrame;
use kernel::{exit_qemu, serial_print, serial_println, QemuExitCode};
use lazy_static::lazy_static;

bootloader_api::entry_point!(test_main);

pub const PRIMARY_PIC_OFFSET: u8 = 104;
pub const SECONDARY_PIC_OFFSET: u8 = 112;
pub static PICS: spin::Mutex<Pics> =
    spin::Mutex::new(unsafe { Pics::new(PRIMARY_PIC_OFFSET, SECONDARY_PIC_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.add_interrupt_handler(
            IdtIndex::TimerInterruptIndex,
            kernel::handler!(timer_interrupt_handler),
        );

        idt
    };
}

extern "C" fn timer_interrupt_handler(_stack_frame: &ExceptionStackFrame) {
    crate::serial_print!("\nTimer Interrupt Handler");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(IdtIndex::TimerInterruptIndex as u8);
    }
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    kernel::hlt()
}

fn test_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kernel::serial_println!("Divide By Zero Error Test");

    kernel::interrupts::testonly_gdt_init();
    IDT.load();

    unsafe {
        log::info!("Initialize Chained PICs");
        PICS.lock().init();
    }

    serial_print!("Starting timer interrupts");
    kernel::interrupts::enable_hardware_interrupts();

    kernel::hlt();

    #[allow(unreachable_code)]
    {
        serial_println!("[Test did not call timer interrupt handler]");
        exit_qemu(QemuExitCode::Failed);
        kernel::hlt()
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info);
}
