use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct PageTableFlags : u64 {
        // Indicates if the page is currently in memory
        const PRESENT = 1;
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
