use crate::memory::paddr::PhysicalAddress;

use core::ops::RangeInclusive;

// x86 uses a frame size of 4KB
pub const FRAME_SIZE: u64 = 4096;

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct FrameRange {
    start_frame: Frame,
    end_frame: Frame,
    // Indicates if the end frame is part of the range
    is_inclusive: bool,
}

impl FrameRange {
    pub fn new(start_frame: Frame, end_frame: Frame, is_inclusive: bool) -> FrameRange {
        if (start_frame.start_address.address() + FRAME_SIZE) >= end_frame.start_address.address() {
            panic!("Start Frame overlaps with end frame");
        }

        FrameRange {
            start_frame,
            end_frame,
            is_inclusive,
        }
    }

    pub fn start_frame(&self) -> Frame {
        self.start_frame
    }

    pub fn end_frame(&self) -> Frame {
        self.end_frame
    }

    pub fn is_inclusive(&self) -> bool {
        self.is_inclusive
    }

    pub fn num_frames(&self) -> u64 {
        let num_frames: u64 = (self.end_frame.start_address.address()
            - self.start_frame.start_address.address())
            / FRAME_SIZE;

        if self.is_inclusive {
            return num_frames + 1;
        } else {
            return num_frames;
        }
    }

    pub fn address_range(&self) -> RangeInclusive<u64> {
        if self.is_inclusive {
            return self.start_frame.start_address.address()
                ..=(self.end_frame.start_address.address() + FRAME_SIZE - 1);
        } else {
            return self.start_frame.start_address.address()
                ..=(self.end_frame.start_address.address() - 1);
        }
    }
}

#[test_case]
fn test_frame_creation_is_successful() {
    let paddr = PhysicalAddress::new(0x18);
    let frame = Frame::new(paddr);

    assert_eq!(frame.start_address().address(), 0x18);
    assert_eq!(frame.size(), FRAME_SIZE);
}

#[test_case]
fn test_frame_range_creation_is_successful() {
    let paddr1 = PhysicalAddress::new(0x18);
    let paddr2 = paddr1 + 3 * FRAME_SIZE;

    let start_frame = Frame::new(paddr1);
    let end_frame = Frame::new(paddr2);

    let frame_range = FrameRange::new(start_frame, end_frame, /*is_inclusive*/ false);
    assert_eq!(frame_range.num_frames(), 3);
    assert_eq!(frame_range.address_range(), 0x18..=0x3017);

    let inlusive_frame_range = FrameRange::new(start_frame, end_frame, /*is_inclusive*/ true);
    assert_eq!(inlusive_frame_range.num_frames(), 4);
    assert_eq!(inlusive_frame_range.address_range(), 0x18..=0x4017);
}
