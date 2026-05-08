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

        // Indiactes if the page maps to a huge page instead of a PagetTable
        const HUGE_PAGE = 1 << 7;

        // Indicates the mapping isn't flushed from the TLB when the address space is switched
        const GLOBAL = 1 << 8;

        // Forbids execution from mapped frames
        const NO_EXECUTE = 1 << 63;
    }
}

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

const PAGE_TABLE_SIZE: usize = 512;

#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_SIZE],
}
