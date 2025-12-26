// The no_std attribute allows us to Rust code on the bare metal without an
// underlying operating system. This means we cannot use threads, files, heap
// memory, the network, random numbers, standard output, or any other features
// requiring OS abstractions or specific hardware.
#![no_std]

// In a typical Rust binary that links the standard library, execution starts
// in a C runtime library that creates a stack and places the arguments in the
// right registers. Post this, the runtime calls the main function.
//
// Freestanding Rust executables do not have access to the Rust runtime. Hence,
// we need to remove the main function and provide a new entry point
// (the _start function defined below).
#![no_main]

use core::panic::PanicInfo;

static VGA_BUFFER_ADDRESS: usize = 0xb8000;

// This function is called on panic.
//
// A by product of not using the standard library, is that we have no panic
// handlers. The panic handler will never return and this is indicated by
// returning the never type (!).
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Rust uses name mangling by default. Name mangling is the process of giving
// every function a unique name. We do not want the Rust compiler to change the
// name of the _start function. This is required to let the linker know of the
// entry point.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let message: &[u8] = b"Toy Rust Kernel!";
    let vga_buffer: *mut u8 = VGA_BUFFER_ADDRESS as *mut u8;


    for (i, &byte) in message.iter().enumerate() {
        unsafe {
            // Each VGA buffer character contains an ASCII and a color byte.
            *vga_buffer.offset(i as isize * 2) = byte;
            // Green (0x2) -> http://fountainware.com/EXPL/vga_color_palettes.htm
            *vga_buffer.offset(i as isize * 2 + 1) = 0x2;
        }
    }

    loop {}
}
