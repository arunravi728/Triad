#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::memory::vaddr::VirtualAddress;
use kernel::{exit_qemu, serial_print, serial_println, QemuExitCode};

bootloader_api::entry_point!(test_main);

fn test_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    test_invalid_subtraction_panics();
    serial_println!("[Test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    kernel::hlt()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    kernel::hlt()
}

fn test_invalid_subtraction_panics() {
    serial_print!("test_invalid_subtraction_panics...\t");
    let vaddr = VirtualAddress::new(0x0A);
    let _ = vaddr - 0x0B as u64;
}
