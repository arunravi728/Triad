// This file contains the various interrupt handlers used by the IDT

pub(crate) extern "C" fn divide_error_handler() -> ! {
    crate::println!("DIVIDE ERROR INTERRUPT");
    loop {}
}
