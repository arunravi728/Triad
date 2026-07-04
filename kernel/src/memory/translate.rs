use core::ops::RangeInclusive;

use crate::memory::page_table::{PAGE_TABLE_INDEX_LENGTH, PAGE_TABLE_OFFSET_LENGTH};
use crate::memory::{
    frame::Frame, paddr::PhysicalAddress, page_table::PageTable, page_table::PageTableEntry,
    page_table::PageTableFlags, vaddr::VirtualAddress,
};
use crate::registers::control::CR3;
const PAGE_TABLE_LEVELS: RangeInclusive<u16> = 1..=4;

#[inline]
pub fn translate(vaddr: VirtualAddress, paddr_offset: u64) -> Option<PhysicalAddress> {
    let (level_4_page_table_frame, _) = CR3::read();
    let mut page_table_frame: Frame = level_4_page_table_frame;

    for level in PAGE_TABLE_LEVELS.rev() {
        let page_table: &PageTable =
            unsafe { &*get_page_table_ptr(paddr_offset, page_table_frame.start_address()) };

        let pte: &PageTableEntry = &page_table[vaddr.page_table_index(level) as usize];
        page_table_frame = match pte.frame() {
            Some(frame) => frame,
            None => return None,
        };

        // If the HUGE_PAGE bit is set, it means the page table walk stops early and this entry
        // points directly to a large contiguous block of physical memory (e.g. 2MB or 1GB).
        if pte.flags().contains(PageTableFlags::HUGE_PAGE) {
            // A standard page uses 12 bits for the offset (4KB).
            // Each skipped page table level turns its 9 index bits into part of the offset.
            // Level 2 (2MB): 12 + 9 = 21 bits. Level 3 (1GB): 12 + 18 = 30 bits.
            let huge_page_offset_mask: u64 =
                (1 << (PAGE_TABLE_OFFSET_LENGTH + (level - 1) * PAGE_TABLE_INDEX_LENGTH)) - 1;

            return Some(PhysicalAddress::new(
                page_table_frame.start_address().address()
                    + (vaddr.address() & huge_page_offset_mask),
            ));
        }
    }

    Some(PhysicalAddress::new(
        page_table_frame.start_address().address() + vaddr.page_table_offset() as u64,
    ))
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
pub fn get_page_table_ptr(paddr_offset: u64, page_table_paddr: PhysicalAddress) -> *mut PageTable {
    let page_table_vaddr: VirtualAddress =
        VirtualAddress::new(page_table_paddr.address() + paddr_offset);
    page_table_vaddr.address() as *mut PageTable
}
