// Support for two chained Intel 8259 PICs. The programmable interrupt controller (PIC) is a
// chip used to route hardware interrupts to the CPU. More info can be found at
// http://wiki.osdev.org/PIC.

#![no_std]

use x86_64::instructions::port::Port;

struct Pic {
    idt_base_offset: u8,

    command_port: Port<u8>,

    data_port: Port<u8>,
}
