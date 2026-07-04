use crate::memory::paddr::PhysicalAddress;

use core::ops::RangeInclusive;

// x86 uses a frame size of 4KB
pub const FRAME_SIZE: u64 = 4096;

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Frame {
    start_address: PhysicalAddress,
}

impl Frame {
    // Returns a frame whose start address is the largest that is less than
    // the address provided.
    #[inline]
    pub fn new(paddr: PhysicalAddress) -> Frame {
        let address = PhysicalAddress::new(paddr.address() & !(FRAME_SIZE - 1));
        Frame {
            start_address: address,
        }
    }

    #[inline]
    pub fn start_address(&self) -> PhysicalAddress {
        self.start_address
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
    #[inline]
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

    #[inline]
    pub fn start_frame(&self) -> Frame {
        self.start_frame
    }

    #[inline]
    pub fn end_frame(&self) -> Frame {
        self.end_frame
    }

    #[inline]
    pub fn is_inclusive(&self) -> bool {
        self.is_inclusive
    }

    #[inline]
    pub fn num_frames(&self) -> u64 {
        let num_frames: u64 = (self.end_frame.start_address.address()
            - self.start_frame.start_address.address())
            / FRAME_SIZE;

        if self.is_inclusive {
            return num_frames + 1;
        }

        return num_frames;
    }

    #[inline]
    pub fn address_range(&self) -> RangeInclusive<u64> {
        if self.is_inclusive {
            return self.start_frame.start_address.address()
                ..=(self.end_frame.start_address.address() + FRAME_SIZE - 1);
        }

        return self.start_frame.start_address.address()
            ..=(self.end_frame.start_address.address() - 1);
    }
}

#[test_case]
fn test_frame_creation_is_successful() {
    let paddr = PhysicalAddress::new(0x18);
    let frame = Frame::new(paddr);

    assert_eq!(frame.start_address().address(), 0x00);
}

#[test_case]
fn test_frame_range_creation_is_successful() {
    let paddr1 = PhysicalAddress::new(0x18);
    let paddr2 = paddr1 + 3 * FRAME_SIZE;

    let start_frame = Frame::new(paddr1);
    let end_frame = Frame::new(paddr2);

    let frame_range = FrameRange::new(start_frame, end_frame, /*is_inclusive*/ false);
    assert_eq!(frame_range.num_frames(), 3);
    assert_eq!(frame_range.address_range(), 0x00..=(3 * FRAME_SIZE - 1));

    let inlusive_frame_range = FrameRange::new(start_frame, end_frame, /*is_inclusive*/ true);
    assert_eq!(inlusive_frame_range.num_frames(), 4);
    assert_eq!(
        inlusive_frame_range.address_range(),
        0x00..=(4 * FRAME_SIZE - 1)
    );
}

#[test_case]
fn test_frame_creation_with_in_between_address_successful() {
    let paddr = PhysicalAddress::new(2 * FRAME_SIZE - 1);
    let frame = Frame::new(paddr);

    assert_eq!(frame.start_address.address(), FRAME_SIZE);
}
