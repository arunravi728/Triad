#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::{exit_qemu, serial_print, serial_println, QemuExitCode};

bootloader_api::entry_point!(test_main);

fn test_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    should_panic();
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

fn should_panic() {
    serial_print!("test_should_panic...");
    assert_eq!(0, 1);
}
