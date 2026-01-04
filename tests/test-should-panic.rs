#![no_std]
#![no_main]

use core::panic::PanicInfo;
use triad::{exit_qemu, serial_print, serial_println, QemuExitCode};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    should_panic();
    serial_println!("[Test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

fn should_panic() {
    serial_print!("test_should_panic...");
    assert_eq!(0, 1);
}
