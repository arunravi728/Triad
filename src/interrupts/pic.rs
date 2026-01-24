// Support for two chained Intel 8259 PICs. The programmable interrupt controller (PIC) is a
// chip used to route hardware interrupts to the CPU. More info can be found at
// http://wiki.osdev.org/PIC.

use x86_64::instructions::port::Port;

const CMD_EOI: u8 = 0x20;

struct Pic {
    idt_base_offset: u8,
    command_port: Port<u8>,
    data_port: Port<u8>,
}

impl Pic {
    unsafe fn end_interrupt(&mut self) {
        self.command_port.write(CMD_EOI);
    }

    unsafe fn read_mask(&mut self) -> u8 {
        self.data_port.read()
    }

    unsafe fn write_mask(&mut self, mask: u8) {
        self.data_port.write(mask);
    }
}
