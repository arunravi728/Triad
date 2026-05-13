use bitflags::bitflags;

bitflags! {
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

// On x86_64 machines, the page table entry is 8 bytes large.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    pub fn new() -> PageTableEntry {
        PageTableEntry { entry: 0 }
    }

    pub fn is_unused(&self) -> bool {
        self.entry == 0
    }

    pub fn set_unused(&mut self) {
        self.entry = 0;
    }
}

// On x86_64 machines, each page table can have a maximum of 512 entries.
const PAGE_TABLE_SIZE: usize = 512;

#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_SIZE],
}
