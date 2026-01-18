// There are two types of GDT entries in long mode: user and system segment descriptors.
// Descriptors for code and data segment segments are user segment descriptors.
//
// System descriptors such as TSS descriptors are contain a base address and a limit. Therefore,
// system segments are 128 bits. They are stored as two consecutive entries in the GDT.
pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

// The Global Descriptor Table (GDT) was used for memory segmentation. Segmentation is not widely
// used in machines anymore as we use paging. The GDT is thus used on 64-bit machines for
// user/kernel mode switching and loading the TSS.
pub struct Gdt {
    table: [u64; 8],
    len: usize,
}

impl Gdt {
    pub fn new() -> Gdt {
        Gdt {
            table: [0; 8],
            // The first entry of the GDT should always be NULL (0), hence we initialize the len to
            // be 0.
            len: 1,
        }
    }
}
