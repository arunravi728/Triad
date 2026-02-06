pub fn run_without_interrupts<Callback, Return>(callback: Callback) -> Return
where
    Callback: FnOnce() -> Return,
{
    let interrupts_enabled: bool = are_interrupts_enabled();

    if interrupts_enabled {
        disable_interrupts();
    }

    let ret = callback();

    if interrupts_enabled {
        enable_interrupts();
    }

    ret
}

#[inline]
pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti", options(preserves_flags, nostack));
    }
}

#[inline]
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cli", options(preserves_flags, nostack));
    }
}

#[inline]
pub fn are_interrupts_enabled() -> bool {
    let rflags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) rflags,
            options(nomem, preserves_flags)
        );
    }
    (rflags & (1 << 9)) != 0
}
