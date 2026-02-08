// Kernel rings are security feature which provides a protection layer for programs. Each ring
// offers a varied degree of access to system resources.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KernelRings {
    // Ring 0 corresponds to the highest level of privilege (kernel mode). This Ring offers the
    // most access to resources. When starting up, the OS runs in this mode unless it switches out.
    // Interrupt handlers run in this mode.
    Ring0 = 0,

    // Rings 1 and 2 are generally used for device-drivers. They offer less access than Ring0, but
    // more than Ring 3.
    Ring1 = 1,
    Ring2 = 2,

    // Ring 3 corresponds to the lowest level of privilege (user mode). This Ring offers the least
    // access to resources. This is where user-space programs run.
    Ring3 = 3,
}

impl KernelRings {
    pub fn new(privilege_level: u16) -> Self {
        match privilege_level {
            0 => KernelRings::Ring0,
            1 => KernelRings::Ring1,
            2 => KernelRings::Ring2,
            3 => KernelRings::Ring3,
            _ => {
                panic!("Illegal Privilege Level")
            }
        }
    }
}
