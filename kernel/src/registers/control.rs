use crate::memory::vaddr::VirtualAddress;

use core::arch::asm;

// This register holds the value called Page Fault Linear Address (PFLA).
// When a page fault occurs, the address the program attempted to access is stored in the CR2 register.
#[derive(Debug)]
pub struct CR2;

impl CR2 {
    #[inline]
    pub fn read() -> VirtualAddress {
        let mut value: u64 = 0;
        unsafe {
            asm!(
                "mov {}, cr2",
                out(reg) value,
                options(nomem, nostack, preserves_flags)
            );
        }

        VirtualAddress::new(value)
    }
}
