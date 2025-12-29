#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(triad::test_runner)]
#![reexport_test_harness_main = "run_test"]

use core::panic::PanicInfo;
use triad::{println, test_panic_handler};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    run_test();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

#[test_case]
fn test_boot() {
    println!("test_println output");
}
