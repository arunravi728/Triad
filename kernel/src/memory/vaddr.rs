use core::ops::{Add, Sub};

// A 64 bit virtual address
//
// On x86_64 machines, only the lower 48 bits can be used. This is because x86_64 machines only
// support 4 level page tables. Bits 48 - 63 is set via sign extension of the 48th bit.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    #[inline]
    pub fn new(addr: u64) -> VirtualAddress {
        // addr << 16 --> Moves the 48th bit to the MSB
        // Right shifting an i64 leads to two's complement MSB sign extension
        // eg:- 0000x000 << 4 --> x0000000 as i64 >> 4 --> xxxxx000
        VirtualAddress(((addr << 16) as i64 >> 16) as u64)
    }

    #[inline]
    pub fn zero() -> VirtualAddress {
        VirtualAddress(0x00)
    }

    #[inline]
    pub fn address(&self) -> u64 {
        self.0
    }
}

// self + arg
impl<T: Into<u64>> Add<T> for VirtualAddress {
    type Output = VirtualAddress;
    #[inline]
    fn add(self, arg: T) -> Self::Output {
        VirtualAddress::new(self.address() + arg.into())
    }
}

// self + arg
impl Add<VirtualAddress> for VirtualAddress {
    type Output = VirtualAddress;
    #[inline]
    fn add(self, arg: VirtualAddress) -> Self::Output {
        VirtualAddress::new(self.address() + arg.address())
    }
}

// self - arg
impl<T: Into<u64> + Clone> Sub<T> for VirtualAddress {
    type Output = VirtualAddress;
    #[inline]
    fn sub(self, arg: T) -> Self::Output {
        if self.address() < arg.clone().into() {
            panic!("Can't subtract a larger address from a smaller one'");
        }

        VirtualAddress::new(self.address() - arg.into())
    }
}

// self - arg
impl Sub<VirtualAddress> for VirtualAddress {
    type Output = VirtualAddress;
    #[inline]
    fn sub(self, arg: VirtualAddress) -> Self::Output {
        if self.address() < arg.address() {
            panic!("Can't subtract a larger address from a smaller one'");
        }

        VirtualAddress::new(self.address() - arg.address())
    }
}

#[test_case]
fn test_virtual_address_creation_successful() {
    let zero = VirtualAddress::zero();
    assert_eq!(zero.address(), 0x00);

    let addr = VirtualAddress::new(0x18);
    assert_eq!(addr.address(), 0x18);
}

#[test_case]
fn test_virtual_address_only_uses_48_bits() {
    let address: u64 = 0xFFFF700000000000;
    let vaddr = VirtualAddress::new(address);
    let expected_address: u64 = 0x700000000000;
    assert_eq!(vaddr.address(), expected_address);
    assert_ne!(vaddr.address(), address);
    assert_eq!(vaddr.address(), (((address << 16) as i64 >> 16) as u64));
}

#[test_case]
fn test_basic_add_is_successful() {
    let vaddr = VirtualAddress::new(0x42);
    let new_vaddr = vaddr + 0x06 as u64;
    assert_eq!(new_vaddr.address(), 0x48);
}

#[test_case]
fn test_adding_virtual_addresses_is_successful() {
    let a = VirtualAddress::new(0x42);
    let b = VirtualAddress::new(0x06);
    let c = a + b;
    assert_eq!(c.address(), 0x48);
}

#[test_case]
fn test_basic_subtraction_is_successful() {
    let vaddr = VirtualAddress::new(0x0A);
    let new_vaddr = vaddr - 0x06 as u64;
    assert_eq!(new_vaddr.address(), 0x04);
}

#[test_case]
fn test_subtracting_virtual_addresses_is_successful() {
    let a = VirtualAddress::new(0x0A);
    let b = VirtualAddress::new(0x06);
    let c = a - b;
    assert_eq!(c.address(), 0x04);
}
