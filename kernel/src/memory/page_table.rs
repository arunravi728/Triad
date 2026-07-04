use bitflags::bitflags;
use core::fmt;

use crate::memory::{paddr::PhysicalAddress, vaddr::VirtualAddress};

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct PageFaultErrorCodes: u64 {
        const PAGE_PROTECTION_VIOLATION = 1;
        const WRITE_VIOLATION = 1 << 1;
        const UNPRIVILEGED_USER = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
        const PROTECTION_KEY = 1 << 5;
        const SHADOW_STACK = 1 << 6;
        const HUGE_PAGE = 1 << 7;
    }
}

bitflags! {
    #[allow(dead_code)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct PageTableFlags : u64 {
        // Indicates if the page is currently in memory
        const PRESENT = 1;

        // Indicates if the page is writable
        const WRITABLE = 1 << 1;

        // Indicates if a page is user accessible
        const USER_ACCESSIBLE = 1 << 2;

        // Indicates if the write is to memory directly bypassing the cache
        const WRITE_THROUGH_CACHING = 1 << 3;

        // Disables caching
        const DISABLE_CACHING = 1 << 4;

        // Indicates if the page is accessed
        const ACCESSED = 1 << 5;

        // Indicates if the mapped frame is written to
        const DIRTY = 1 << 6;

        // Indicates if the page maps to a huge page instead of a Page Table
        const HUGE_PAGE = 1 << 7;

        // Indicates the mapping isn't flushed from the TLB when the address space is switched
        const GLOBAL = 1 << 8;

        // Bits 9 - 11 can be freely used by the OS
        // Bits 12 - 51 are used to represent the frame physical address
        // Bits 52 - 62 can be freely used by the OS
        // This is possible because we always point to a 4096-byte aligned address.
        // This means that bits 0–11 are always zero.
        // The same is true for bits 52–63, x86_64 only supports 52-bit physical addresses.

        // Forbids execution from mapped frames
        const NO_EXECUTE = 1 << 63;
    }
}

const PTE_PADDR_MASK: u64 = 0x000f_ffff_ffff_f000u64;

// On x86_64 machines, the page table entry is 8 bytes large.
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PageTableEntry {
    entry: u64,
}

#[allow(dead_code)]
impl PageTableEntry {
    #[inline]
    pub fn new() -> PageTableEntry {
        PageTableEntry { entry: 0 }
    }

    #[inline]
    pub fn is_unused(&self) -> bool {
        self.entry == 0
    }

    #[inline]
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    #[inline]
    pub fn paddr(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.entry & PTE_PADDR_MASK)
    }

    #[inline]
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_retain(self.entry & !PTE_PADDR_MASK)
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("paddr", &self.paddr());
        f.field("flags", &self.flags());
        f.finish()
    }
}

// On x86_64 machines, each page table can have a maximum of 512 entries.
const PTE_COUNT: usize = 512;

#[allow(dead_code)]
#[repr(align(4096))]
pub struct PageTable([PageTableEntry; PTE_COUNT]);

impl PageTable {
    #[inline]
    pub fn new() -> PageTable {
        PageTable([PageTableEntry::new(); PTE_COUNT])
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        (0..PTE_COUNT).map(move |i| &self.0[i])
    }
}

// The kernel runs on paging. This means all the addresses obtained from registers like CR3 are
// virtual addresses. In order to access the underlying physical memory pointed to by PageTables,
// we need a way to map the physical addresses to virtual addresses.
//
// In this kernel we map all of the physical memory space to the virtual memory space. The formula
// to calculate the virtual address given the page table physical address is as follows -
//
// virtual_address = physical_address + physical_address_offset
//
// This essentially means that the given physical_address can be accessed via the computed
// virtual_address
#[inline]
pub fn get_page_table_ptr(
    paddr_offset: u64,
    page_table_paddr: PhysicalAddress,
) -> &'static mut PageTable {
    let page_table_vaddr: VirtualAddress =
        VirtualAddress::new(page_table_paddr.address() + paddr_offset);

    let page_table_ptr: *mut PageTable = page_table_vaddr.address() as *mut PageTable;

    unsafe { &mut *page_table_ptr }
}

#[test_case]
fn test_page_table_iterator() {
    let page_table = PageTable::new();

    let mut test_pti: usize = 0;
    let test_pte: PageTableEntry = PageTableEntry::new();

    for (pti, pte) in page_table.iter().enumerate() {
        assert_eq!(pti, test_pti);
        assert_eq!(*pte, test_pte);
        test_pti += 1;
    }

    assert_eq!(test_pti, PTE_COUNT as usize);
}
