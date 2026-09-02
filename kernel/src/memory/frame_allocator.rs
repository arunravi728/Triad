use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

use crate::memory::{frame::Frame, paddr::PhysicalAddress};

#[cfg(test)]
use crate::memory::frame::FRAME_SIZE;
#[cfg(test)]
use bootloader_api::info::MemoryRegion;

pub struct FrameAllocator {
    memory_regions: &'static MemoryRegions,
    next_frame_number: usize,
}

impl FrameAllocator {
    // It is the callers responsibility to ensure that the passed memory regions are valid.
    // All frames that are marked usable are unused. We will pass the memory regions part
    // that is provided by the Bootloader in the BootInfo struct.
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        FrameAllocator {
            memory_regions,
            next_frame_number: 0,
        }
    }

    pub fn allocate(&mut self) -> Option<Frame> {
        let frame = self.get_usable_frames().nth(self.next_frame_number);
        self.next_frame_number += 1;
        frame
    }

    fn get_usable_frames(&self) -> impl Iterator<Item = Frame> {
        self.memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .map(|r| r.start..r.end)
            .flat_map(|r| r.step_by(4096))
            .map(|addr| Frame::new(PhysicalAddress::new(addr)))
    }
}

#[test_case]
#[allow(static_mut_refs)]
fn test_base_usable_memory_region() {
    static mut REGIONS: [MemoryRegion; 1] = [MemoryRegion {
        start: 0,
        end: 3 * FRAME_SIZE,
        kind: MemoryRegionKind::Usable,
    }];
    static mut STORAGE: Option<MemoryRegions> = None;

    let memory_regions: &'static MemoryRegions = unsafe {
        STORAGE = Some((&mut REGIONS[..]).into());
        STORAGE.as_ref().unwrap()
    };

    let mut allocator = unsafe { FrameAllocator::init(memory_regions) };

    assert_eq!(allocator.allocate().unwrap().start_address().address(), 0);
    assert_eq!(
        allocator.allocate().unwrap().start_address().address(),
        FRAME_SIZE
    );
    assert_eq!(
        allocator.allocate().unwrap().start_address().address(),
        2 * FRAME_SIZE
    );
    assert!(allocator.allocate().is_none());
}
