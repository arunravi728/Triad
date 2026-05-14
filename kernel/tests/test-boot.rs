#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "run_test"]

use core::panic::PanicInfo;
use kernel::test_panic_handler;

bootloader_api::entry_point!(test_main);

fn test_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    run_test();
    kernel::hlt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

#[test_case]
fn test_boot() {
    kernel::serial_println!("test_println output");
}
