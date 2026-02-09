use crate::interrupts::privilege::KernelRings;
use crate::interrupts::segment::SegmentSelector;
use crate::interrupts::tss::TaskStateSegment;

use bitflags::bitflags;
use core::arch::asm;

use x86_64::addr::VirtAddr;
use x86_64::instructions::tables::DescriptorTablePointer;

// This command helps load an GDT. The commands stores the active GDT and its length. The lgdt
// instruction expects a pointer to a data structure holding the start address of the GDT and its
// length.
#[inline]
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
    }
}

// The following constants are used by the various Descriptors as flags.
bitflags! {
    struct DescriptorFlags: u64 {
        const WRITABLE          = 1 << 41;
        const CONFORMING        = 1 << 42;
        const EXECUTABLE        = 1 << 43;
        const USER_SEGMENT      = 1 << 44;
        const PRESENT           = 1 << 47;
        const LONG_MODE         = 1 << 53;
    }
}

// There are two types of GDT entries in long mode: user and system segment descriptors.
// Descriptors for code and data segment segments are user segment descriptors.
//
// System descriptors such as TSS descriptors are contain a base address and a limit. Therefore,
// system segments are 128 bits. They are stored as two consecutive entries in the GDT.
pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    pub fn kernel_code_segment() -> Descriptor {
        let flags = DescriptorFlags::USER_SEGMENT
            | DescriptorFlags::PRESENT
            | DescriptorFlags::EXECUTABLE
            | DescriptorFlags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }

    pub fn kernel_data_segment() -> Descriptor {
        let flags = DescriptorFlags::USER_SEGMENT
            | DescriptorFlags::PRESENT
            | DescriptorFlags::WRITABLE;
        Descriptor::UserSegment(flags.bits())
    }

    // The layout of the TSS Segment can be found at https://os.phil-opp.com/double-faults/#tss-segments
    // We require the 'static lifetime for the TaskStateSegment reference, since the hardware might
    // access it on every interrupt as long as the OS runs.
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        use bit_field::BitField;
        use core::mem::size_of;

        let ptr = tss as *const _ as u64;

        let mut low = DescriptorFlags::PRESENT.bits();

        // base
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));

        // limit (the `-1` in needed since the bound is inclusive)
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);

        // type (0b1001 = available 64-bit tss)
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}

// The Global Descriptor Table (GDT) was used for memory segmentation. Segmentation is not widely
// used in machines anymore as we use paging. The GDT is thus used on 64-bit machines for
// user/kernel mode switching and loading the TSS.
pub struct GlobalDescriptorTable {
    table: [u64; 8],
    len: usize,
}

impl GlobalDescriptorTable {
    pub fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            // The first entry of the GDT should always be NULL (0), hence we initialize the len to
            // be 0.
            len: 1,
        }
    }

    pub fn load(&'static self) {
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: VirtAddr::new(self.table.as_ptr() as u64),
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };

        unsafe { lgdt(&ptr) };
    }

    pub fn add(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(val) => self.push(val),
            Descriptor::SystemSegment(low, high) => {
                let index = self.push(low);
                self.push(high);
                index
            }
        };
        SegmentSelector::new(index as u16, KernelRings::Ring0)
    }

    fn push(&mut self, value: u64) -> usize {
        if self.len > self.table.len() {
            panic!("GDT FULL");
        }

        self.table[self.len] = value;
        self.len += 1;
        self.len - 1
    }
}
