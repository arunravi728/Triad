use core::ops::RangeInclusive;

use crate::memory::vaddr::VirtualAddress;

// x86 uses a page size of 4KB
pub const PAGE_SIZE: u64 = 4096;

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Page {
    start_address: VirtualAddress,
}

impl Page {
    pub fn new(start_address: VirtualAddress) -> Page {
        Page { start_address }
    }

    // Returns a frame whose start address is the largest that is less than
    // the address provided.
    pub fn with_address(paddr: VirtualAddress) -> Page {
        let address = VirtualAddress::new(paddr.address() & !(PAGE_SIZE - 1));
        Page {
            start_address: address,
        }
    }

    pub fn start_address(&self) -> VirtualAddress {
        self.start_address
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct PageRange {
    start_page: Page,
    end_page: Page,
    is_inclusive: bool,
}

impl PageRange {
    pub fn new(start_page: Page, end_page: Page, is_inclusive: bool) -> PageRange {
        if (start_page.start_address.address() + PAGE_SIZE) >= end_page.start_address.address() {
            panic!("Start Page overlaps with end frame");
        }

        PageRange {
            start_page,
            end_page,
            is_inclusive,
        }
    }

    pub fn start_page(&self) -> Page {
        self.start_page
    }

    pub fn end_page(&self) -> Page {
        self.end_page
    }

    pub fn is_inclusive(&self) -> bool {
        self.is_inclusive
    }

    pub fn num_pages(&self) -> u64 {
        let num_pages = (self.end_page.start_address.address()
            - self.start_page.start_address.address())
            / PAGE_SIZE;

        if self.is_inclusive {
            return num_pages + 1;
        }

        return num_pages;
    }

    pub fn address_range(&self) -> RangeInclusive<u64> {
        if self.is_inclusive {
            return self.start_page.start_address.address()
                ..=(self.end_page.start_address.address() + PAGE_SIZE - 1);
        }

        return self.start_page.start_address.address()
            ..=(self.end_page.start_address.address() - 1);
    }
}

#[test_case]
fn test_page_creation_is_successful() {
    let vaddr = VirtualAddress::new(0x18);
    let page = Page::new(vaddr);

    assert_eq!(page.start_address.address(), 0x18);
}

#[test_case]
fn test_page_range_creation_is_successful() {
    let vaddr1 = VirtualAddress::new(0x18);
    let vaddr2 = vaddr1 + 3 * PAGE_SIZE;

    let start_page = Page::new(vaddr1);
    let end_page = Page::new(vaddr2);

    let page_range = PageRange::new(start_page, end_page, /*is_inclusive*/ false);
    assert_eq!(page_range.num_pages(), 3);
    assert_eq!(page_range.address_range(), 0x18..=0x3017);

    let inlusive_page_range = PageRange::new(start_page, end_page, /*is_inclusive*/ true);
    assert_eq!(inlusive_page_range.num_pages(), 4);
    assert_eq!(inlusive_page_range.address_range(), 0x18..=0x4017);
}

#[test_case]
fn test_page_creation_with_in_between_address_successful() {
    let vaddr = VirtualAddress::new(2 * PAGE_SIZE - 1);
    let frame = Page::with_address(vaddr);

    assert_eq!(frame.start_address.address(), PAGE_SIZE);
}
