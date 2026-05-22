use core::{
    fmt,
    ops::{Add, Sub},
};

// A 64 bit physical address
//
// On x86_64 machines, only the lower 52 bits can be used.
// The top 12 bits need to be zero.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    #[inline]
    pub fn new(addr: u64) -> PhysicalAddress {
        PhysicalAddress(addr & !(0xFFF0 << 48))
    }

    #[inline]
    pub const fn zero() -> PhysicalAddress {
        PhysicalAddress(0x00)
    }

    #[inline]
    pub fn address(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_ptr<T: Sized>(ptr: *const T) -> Self {
        Self::new(ptr as u64)
    }
}

impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VirtAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

// self + arg
impl<T: Into<u64>> Add<T> for PhysicalAddress {
    type Output = PhysicalAddress;
    #[inline]
    fn add(self, arg: T) -> Self::Output {
        PhysicalAddress::new(self.address() + arg.into())
    }
}

// self + arg
impl Add<PhysicalAddress> for PhysicalAddress {
    type Output = PhysicalAddress;
    #[inline]
    fn add(self, arg: PhysicalAddress) -> Self::Output {
        PhysicalAddress::new(self.address() + arg.address())
    }
}

// self - arg
impl<T: Into<u64> + Clone> Sub<T> for PhysicalAddress {
    type Output = PhysicalAddress;
    #[inline]
    fn sub(self, arg: T) -> Self::Output {
        if self.address() < arg.clone().into() {
            panic!("Can't subtract a larger address from a smaller one'");
        }

        PhysicalAddress::new(self.address() - arg.into())
    }
}

// self - arg
impl Sub<PhysicalAddress> for PhysicalAddress {
    type Output = PhysicalAddress;
    #[inline]
    fn sub(self, arg: PhysicalAddress) -> Self::Output {
        if self.address() < arg.address() {
            panic!("Can't subtract a larger address from a smaller one'");
        }

        PhysicalAddress::new(self.address() - arg.address())
    }
}

#[test_case]
fn test_physical_address_creation_successful() {
    let zero = PhysicalAddress::zero();
    assert_eq!(zero.address(), 0x00);

    let addr = PhysicalAddress::new(0x18);
    assert_eq!(addr.address(), 0x18);
}

#[test_case]
fn test_physical_address_creation_from_ptr_successful() {
    let stack: [u8; 4096] = [0; 4096];
    let addr = PhysicalAddress::from_ptr(&raw const stack);
    assert_ne!(addr.address(), 0x00);
}

#[test_case]
fn test_physical_address_only_uses_52_bits() {
    let address: u64 = 0xFFFF700000000000;
    let vaddr = PhysicalAddress::new(address);
    let expected_address: u64 = 0xF700000000000;
    assert_eq!(vaddr.address(), expected_address);
    assert_ne!(vaddr.address(), address);
    assert_eq!(vaddr.address(), (address & !(0xFFF0 << 48)));
}

#[test_case]
fn test_basic_add_is_successful() {
    let vaddr = PhysicalAddress::new(0x42);
    let new_vaddr = vaddr + 0x06 as u64;
    assert_eq!(new_vaddr.address(), 0x48);
}

#[test_case]
fn test_adding_physical_addresses_is_successful() {
    let a = PhysicalAddress::new(0x42);
    let b = PhysicalAddress::new(0x06);
    let c = a + b;
    assert_eq!(c.address(), 0x48);
}

#[test_case]
fn test_basic_subtraction_is_successful() {
    let vaddr = PhysicalAddress::new(0x0A);
    let new_vaddr = vaddr - 0x06 as u64;
    assert_eq!(new_vaddr.address(), 0x04);
}

#[test_case]
fn test_subtracting_physical_addresses_is_successful() {
    let a = PhysicalAddress::new(0x0A);
    let b = PhysicalAddress::new(0x06);
    let c = a - b;
    assert_eq!(c.address(), 0x04);
}
