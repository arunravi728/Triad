use crate::memory::{paddr::PhysicalAddress, page_table::PageTable, vaddr::VirtualAddress};

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
