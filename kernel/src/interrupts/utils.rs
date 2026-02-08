#[allow(dead_code)]
pub fn generate_divide_by_zero_interrupt() {
    unsafe {
        core::arch::asm!(
            "mov dx, 0",
            "div dx",
            out("ax") _,
            out("dx") _,
            options(nomem, nostack)
        );
    }
}

#[allow(dead_code)]
pub fn generate_invalid_opcode_interrupt() {
    unsafe {
        core::arch::asm!("ud2");
    };
}

#[allow(dead_code)]
pub fn generate_breakpoint() {
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    };
}

#[allow(dead_code)]
pub fn generate_page_fault() {
    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    };
}
