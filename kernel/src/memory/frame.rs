use crate::memory::paddr::PhysicalAddress;

// x86 uses a frame size of 4KB
const FRAME_SIZE: u64 = 4096;

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Frame {
    start_address: PhysicalAddress,
    size: u64,
}

impl Frame {
    pub fn new(start_address: PhysicalAddress) -> Frame {
        Frame {
            start_address,
            size: FRAME_SIZE,
        }
    }

    pub fn start_address(&self) -> PhysicalAddress {
        self.start_address
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

#[test_case]
fn test_frame_creation_is_successful() {
    let paddr = PhysicalAddress::new(0x18);
    let frame = Frame::new(paddr);

    assert_eq!(frame.start_address().address(), 0x18);
    assert_eq!(frame.size(), FRAME_SIZE);
}
