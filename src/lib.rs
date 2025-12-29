#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "run_tests"]

use core::panic::PanicInfo;

pub mod serial;
pub mod vga;

// This entry point is for all unit tests belonging to modules linked to lib.rs. We have a separate
// entry point in the main function for all unit tests part of main.rs.
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Calls test_runner()
    run_tests();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// We use the isa-debug-exit device provide by QEMU to exit when running tests. We can quit QEMU
// by writing to the port exposed by the device. The isa-debug-exit device uses port-mapped I/O.
// Whenever a value is written to the ISA_DEBUG_EXIT_PORT, QEMU exits with status (value << 1) | 1.
const ISA_DEBUG_EXIT_PORT: u16 = 0xf4;

// Exit codes are chosen such that they don't clash with the default QEMU exit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    // The operation is unsafe as writing to an I/O port can result in undefined behavior.
    unsafe {
        let mut port = Port::new(ISA_DEBUG_EXIT_PORT);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

// Implement the Testable trait for all types that implement the Fn() trait.
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}
