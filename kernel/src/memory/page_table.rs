use bitflags::bitflags;
use core::fmt;
use core::ops::{Index, IndexMut};

use crate::memory::frame::Frame;
use crate::memory::paddr::PhysicalAddress;

pub const PAGE_TABLE_INDEX_LENGTH: u16 = 9; // Each page table index is 9 bits long
pub const PAGE_TABLE_OFFSET_LENGTH: u16 = 12; // The page table offset is 12 bits long
pub const PAGE_TABLE_OFFSET_MASK: u16 = 0x0FFF;
pub const PTE_PADDR_MASK: u64 = 0x000f_ffff_ffff_f000u64;

// On x86_64 machines, each page table can have a maximum of 512 entries.
pub const PTE_COUNT: usize = 512;

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
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PageTableEntry {
    entry: u64,
}

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

    pub fn set_flags(&mut self, ptf: PageTableFlags) {
        // bitflags lets callers build values with undefined bits via from_bits_retain
        // A PageTableFlags with bits in 12-51 would silently corrupt the frame address.
        // Hence we explicitly negate the frame address bits from ptf.
        self.entry = (self.entry & PTE_PADDR_MASK) | (ptf.bits() & !PTE_PADDR_MASK);
    }

    #[inline]
    pub fn frame(&self) -> Option<Frame> {
        if !self.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        Some(Frame::new(self.paddr()))
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

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
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

#[test_case]
fn test_page_table_constants() {
    assert_eq!(PTE_COUNT, 1usize << PAGE_TABLE_INDEX_LENGTH);
    assert_eq!(
        PAGE_TABLE_OFFSET_MASK as u64,
        (1u64 << PAGE_TABLE_OFFSET_LENGTH) - 1
    );

    // Physical address occupies bits 12..=51.
    assert_eq!(PTE_PADDR_MASK, ((1u64 << 52) - 1) & !((1u64 << 12) - 1));
}

#[test_case]
fn test_page_table_sizes_and_alignment() {
    assert_eq!(core::mem::size_of::<PageTableEntry>(), 8);
    assert_eq!(core::mem::size_of::<PageTable>(), 4096);
    assert_eq!(core::mem::align_of::<PageTable>(), 4096);
}

#[test_case]
fn test_page_table_flag_bit_positions() {
    assert_eq!(PageTableFlags::PRESENT.bits(), 1 << 0);
    assert_eq!(PageTableFlags::WRITABLE.bits(), 1 << 1);
    assert_eq!(PageTableFlags::USER_ACCESSIBLE.bits(), 1 << 2);
    assert_eq!(PageTableFlags::WRITE_THROUGH_CACHING.bits(), 1 << 3);
    assert_eq!(PageTableFlags::DISABLE_CACHING.bits(), 1 << 4);
    assert_eq!(PageTableFlags::ACCESSED.bits(), 1 << 5);
    assert_eq!(PageTableFlags::DIRTY.bits(), 1 << 6);
    assert_eq!(PageTableFlags::HUGE_PAGE.bits(), 1 << 7);
    assert_eq!(PageTableFlags::GLOBAL.bits(), 1 << 8);
    assert_eq!(PageTableFlags::NO_EXECUTE.bits(), 1 << 63);
}

#[test_case]
fn test_page_table_flags_do_not_overlap_address_bits() {
    let all = PageTableFlags::all();
    assert_eq!(all.bits() & PTE_PADDR_MASK, 0);
}

#[test_case]
fn test_pte_new_is_unused() {
    let pte = PageTableEntry::new();
    assert!(pte.is_unused());
    assert_eq!(pte.paddr(), PhysicalAddress::new(0));
    assert_eq!(pte.flags(), PageTableFlags::empty());
    assert!(pte.frame().is_none());
}

#[test_case]
fn test_pte_set_unused_clears_everything() {
    let mut pte = PageTableEntry {
        entry: 0x1000 | PageTableFlags::PRESENT.bits(),
    };
    assert!(!pte.is_unused());

    pte.set_unused();
    assert!(pte.is_unused());
    assert_eq!(pte.flags(), PageTableFlags::empty());
    assert_eq!(pte.paddr(), PhysicalAddress::new(0));
}

#[test_case]
fn test_pte_paddr_ignores_flag_bits() {
    // Low flags (0-11), high flags (52-63), and an address in between.
    let raw = 0xFFF0_0000_0000_0FFFu64 | 0x0000_0000_ABCD_E000;
    let pte = PageTableEntry { entry: raw };
    assert_eq!(pte.paddr(), PhysicalAddress::new(0xABCD_E000));
}

#[test_case]
fn test_pte_paddr_uses_bits_up_to_51() {
    let pte = PageTableEntry {
        entry: PTE_PADDR_MASK,
    };
    assert_eq!(pte.paddr(), PhysicalAddress::new(PTE_PADDR_MASK));
}

#[test_case]
fn test_pte_flags_ignore_address_bits() {
    let raw = 0x0000_0000_ABCD_E000u64
        | PageTableFlags::PRESENT.bits()
        | PageTableFlags::WRITABLE.bits()
        | PageTableFlags::NO_EXECUTE.bits();
    let pte = PageTableEntry { entry: raw };

    assert_eq!(
        pte.flags(),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE
    );
}

#[test_case]
fn test_pte_flags_retain_os_available_bits() {
    // Bits 9-11 and 52-62 are OS-available; flags() should surface them.
    let raw = (1u64 << 9) | (1u64 << 52);
    let pte = PageTableEntry { entry: raw };
    assert_eq!(pte.flags().bits(), raw);
}

#[test_case]
fn test_pte_set_flags_sets_flags() {
    let mut pte = PageTableEntry::new();
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    pte.set_flags(flags);
    assert_eq!(pte.flags(), flags);
    assert!(!pte.is_unused());
}

#[test_case]
fn test_pte_set_flags_preserves_paddr() {
    let mut pte = PageTableEntry {
        entry: 0x0000_0000_1234_5000,
    };
    pte.set_flags(PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE);

    assert_eq!(pte.paddr(), PhysicalAddress::new(0x1234_5000));
    assert_eq!(
        pte.flags(),
        PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE
    );
}

#[test_case]
fn test_pte_set_flags_replaces_old_flags() {
    let mut pte = PageTableEntry::new();
    pte.set_flags(PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::DIRTY);
    pte.set_flags(PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE);

    assert_eq!(
        pte.flags(),
        PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE
    );
}

#[test_case]
fn test_pte_set_flags_empty_clears_flags_keeps_paddr() {
    let mut pte = PageTableEntry {
        entry: 0x0000_0000_00AB_C000 | PageTableFlags::PRESENT.bits(),
    };
    pte.set_flags(PageTableFlags::empty());

    assert_eq!(pte.flags(), PageTableFlags::empty());
    assert_eq!(pte.paddr(), PhysicalAddress::new(0x00AB_C000));
}

#[test_case]
fn test_pte_set_flags_no_execute_roundtrip() {
    let mut pte = PageTableEntry { entry: 0x2000 };
    pte.set_flags(PageTableFlags::NO_EXECUTE);

    assert!(pte.flags().contains(PageTableFlags::NO_EXECUTE));
    assert_eq!(pte.paddr(), PhysicalAddress::new(0x2000));
}

#[test_case]
fn test_pte_set_flags_all_flags_roundtrip() {
    let mut pte = PageTableEntry { entry: 0x7000 };
    pte.set_flags(PageTableFlags::all());

    assert_eq!(pte.flags(), PageTableFlags::all());
    assert_eq!(pte.paddr(), PhysicalAddress::new(0x7000));
}

// Fails against the current implementation; passes with the `& !PTE_PADDR_MASK` fix.
#[test_case]
fn test_pte_set_flags_cannot_clobber_paddr() {
    let mut pte = PageTableEntry {
        entry: 0x0000_0000_1000_0000,
    };
    let bogus = PageTableFlags::from_bits_retain(PTE_PADDR_MASK | PageTableFlags::PRESENT.bits());

    pte.set_flags(bogus);

    assert_eq!(pte.paddr(), PhysicalAddress::new(0x1000_0000));
    assert_eq!(pte.flags(), PageTableFlags::PRESENT);
}

#[test_case]
fn test_pte_frame_none_when_not_present() {
    // Address set but PRESENT clear: no frame.
    let pte = PageTableEntry {
        entry: 0x5000 | PageTableFlags::WRITABLE.bits(),
    };
    assert!(pte.frame().is_none());
}

#[test_case]
fn test_pte_frame_some_when_present() {
    let pte = PageTableEntry {
        entry: 0x5000 | PageTableFlags::PRESENT.bits(),
    };
    assert!(pte.frame().is_some());
}

#[test_case]
fn test_pte_frame_after_set_flags() {
    let mut pte = PageTableEntry { entry: 0x9000 };
    assert!(pte.frame().is_none());

    pte.set_flags(PageTableFlags::PRESENT);
    assert!(pte.frame().is_some());

    pte.set_flags(PageTableFlags::empty());
    assert!(pte.frame().is_none());
}

#[test_case]
fn test_page_table_new_all_unused() {
    let table = PageTable::new();
    assert!(table.iter().all(|pte| pte.is_unused()));
    assert_eq!(table.iter().count(), PTE_COUNT);
}

#[test_case]
fn test_page_table_index_and_index_mut() {
    let mut table = PageTable::new();

    table[0].set_flags(PageTableFlags::PRESENT);
    table[PTE_COUNT - 1].set_flags(PageTableFlags::WRITABLE);

    assert_eq!(table[0].flags(), PageTableFlags::PRESENT);
    assert_eq!(table[PTE_COUNT - 1].flags(), PageTableFlags::WRITABLE);
    assert!(table[1].is_unused());
}

#[test_case]
fn test_page_table_iter_reflects_mutation() {
    let mut table = PageTable::new();
    table[42].set_flags(PageTableFlags::PRESENT);

    let (idx, _) = table
        .iter()
        .enumerate()
        .find(|(_, pte)| !pte.is_unused())
        .unwrap();
    assert_eq!(idx, 42);
}
