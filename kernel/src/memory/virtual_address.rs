// A 64 bit virtual address
//
// On x86_64 machines, only the lower 48 bits can be used. This is because x86_64 machines only
// support 4 level page tables. Bits 48 - 63 is set via sign extension of the 48th bit.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    pub fn new(addr: u64) -> VirtualAddress {
        // addr << 16 --> Moves the 48th bit to the MSB
        // Right shifting an i64 leads to two's complement MSB sign extension
        // eg:- 0000x000 << 4 --> x0000000 as i64 >> 4 --> xxxxx000
        VirtualAddress(((addr << 16) as i64 >> 16) as u64)
    }
}
