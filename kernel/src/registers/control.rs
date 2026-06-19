use crate::memory::frame::Frame;
use crate::memory::paddr::PhysicalAddress;
use crate::memory::vaddr::VirtualAddress;

use bitflags::bitflags;
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

// CR3 enables the processor to translate linear addresses into physical addresses by locating the page
// directory and page tables for the current task. Typically, the upper 20 bits of CR3 become the page
// directory base register (PDBR), which stores the physical address of the first page directory
#[derive(Debug)]
pub struct CR3;

// https://wiki.osdev.org/CPU_Registers_x86#CR3:~:text=Fault%20Linear%20Address-,CR3,-Bit
bitflags! {
    #[derive(Debug, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
    pub struct CR3Flags: u64 {
        const PAGE_LEVEL_WRITE_THROUGH = 1 << 3;
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
    }
}

impl CR3 {
    #[inline]
    pub fn read() -> (Frame, CR3Flags) {
        let mut value: u64 = 0;
        unsafe {
            asm!(
                "mov {}, cr3",
                out(reg) value,
                options(nomem, nostack, preserves_flags)
            );
        }

        let pdbr_mask: u64 = 0x_000F_FFFF_FFFF_F000;
        let cr3_flags_mask: u64 = 0x_0000_0FFF;

        let frame: Frame = Frame::new(PhysicalAddress::new(value & pdbr_mask));
        let flags: CR3Flags = CR3Flags::from_bits_truncate(value & cr3_flags_mask);

        (frame, flags)
    }
}
