#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
// We use a custom test runner defined in this file as opposed to the one defined in lib.rs.
#![test_runner(test_runner)]
#![reexport_test_harness_main = "run_test"]

use core::panic::PanicInfo;
use triad::{QemuExitCode, exit_qemu, serial_println, serial_print};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    run_test();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
        serial_println!("[Test did not panic]");
        exit_qemu(QemuExitCode::Failed);
    }

    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn test_should_panic() {
    serial_print!("test_should_panic...");
    assert_eq!(0, 1);
}






