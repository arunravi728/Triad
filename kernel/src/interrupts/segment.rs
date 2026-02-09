use crate::interrupts::privilege::KernelRings;
use core::arch::asm;

// https://wiki.osdev.org/Segment_Selector
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    #[inline]
    pub const fn new(index: u16, ring: KernelRings) -> Self {
        SegmentSelector((index << 3) | (ring as u16))
    }
}

// Segment registers are 16-bit SegmentSelectors, which index into the GlobalDescriptorTable.
pub trait Segment {
    // Returns the current value of the segment register.
    fn reg() -> SegmentSelector;

    // Reload the segment register.
    unsafe fn set_reg(sel: SegmentSelector);
}

// Code Segment
#[derive(Debug)]
pub struct CS;

impl Segment for CS {
    #[inline]
    fn reg() -> SegmentSelector {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, cs", out(reg) segment, options(nomem, nostack, preserves_flags));
        }
        SegmentSelector(segment)
    }

    #[inline]
    unsafe fn set_reg(sel: SegmentSelector) {
        unsafe {
            asm!(
                "push {sel}",
                "lea {tmp}, [55f + rip]",
                "push {tmp}",
                "retfq",
                "55:",
                sel = in(reg) u64::from(sel.0),
                tmp = lateout(reg) _,
                options(preserves_flags),
            );
        }
    }
}

// Stack Segment
#[derive(Debug)]
pub struct SS;

impl Segment for SS {
    #[inline]
    fn reg() -> SegmentSelector {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, ss", out(reg) segment, options(nomem, nostack, preserves_flags));
        }
        SegmentSelector(segment)
    }

    #[inline]
    unsafe fn set_reg(sel: SegmentSelector) {
        unsafe {
            asm!("mov ss, {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
        }
    }
}

// Data Segment
#[derive(Debug)]
pub struct DS;

impl Segment for DS {
    #[inline]
    fn reg() -> SegmentSelector {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, ds", out(reg) segment, options(nomem, nostack, preserves_flags));
        }
        SegmentSelector(segment)
    }

    #[inline]
    unsafe fn set_reg(sel: SegmentSelector) {
        unsafe {
            asm!("mov ds, {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
        }
    }
}

// Extra Segment
#[derive(Debug)]
pub struct ES;

impl Segment for ES {
    #[inline]
    fn reg() -> SegmentSelector {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, es", out(reg) segment, options(nomem, nostack, preserves_flags));
        }
        SegmentSelector(segment)
    }

    #[inline]
    unsafe fn set_reg(sel: SegmentSelector) {
        unsafe {
            asm!("mov es, {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
        }
    }
}

#[derive(Debug)]
pub struct FS;

impl Segment for FS {
    #[inline]
    fn reg() -> SegmentSelector {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, fs", out(reg) segment, options(nomem, nostack, preserves_flags));
        }
        SegmentSelector(segment)
    }

    #[inline]
    unsafe fn set_reg(sel: SegmentSelector) {
        unsafe {
            asm!("mov fs, {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
        }
    }
}

#[derive(Debug)]
pub struct GS;

impl Segment for GS {
    #[inline]
    fn reg() -> SegmentSelector {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, gs", out(reg) segment, options(nomem, nostack, preserves_flags));
        }
        SegmentSelector(segment)
    }

    #[inline]
    unsafe fn set_reg(sel: SegmentSelector) {
        unsafe {
            asm!("mov gs, {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
        }
    }
}
