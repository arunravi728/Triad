use crate::memory::vaddr::VirtualAddress;

// x86 uses a page size of 4KB
pub const PAGE_SIZE: u64 = 4096;

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Page {
    start_address: VirtualAddress,
    size: u64,
}

impl Page {
    pub fn new(start_address: VirtualAddress) -> Page {
        Page {
            start_address,
            size: PAGE_SIZE,
        }
    }

    pub fn start_address(&self) -> VirtualAddress {
        self.start_address
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

#[test_case]
fn test_page_creation_is_successful() {
    let vaddr = VirtualAddress::new(0x18);
    let page = Page::new(vaddr);

    assert_eq!(page.start_address.address(), 0x18);
    assert_eq!(page.size(), PAGE_SIZE);
}
