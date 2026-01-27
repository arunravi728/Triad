// Support for two chained Intel 8259 PICs. The programmable interrupt controller (PIC) is a
// chip used to route hardware interrupts to the CPU. More info can be found at
// http://wiki.osdev.org/PIC.

use x86_64::instructions::port::Port;

const CMD_EOI: u8 = 0x20;
const CMD_CASCADED_INIT: u8 = 0x11;
const CMD_8086_MODE: u8 = 0x01;
const CMD_DISABLE_PIC: u8 = 0xFF;

const IO_WAIT_PORT: u16 = 0x80;
const PRIMARY_CMD_PORT: u16 = 0x20;
const PRIMARY_DATA_PORT: u16 = 0x21;
const SECONDARY_CMD_PORT: u16 = 0xA0;
const SECONDARY_DATA_PORT: u16 = 0xA1;

pub struct Pics {
    primary: Pic,
    secondary: Pic,
}

impl Pics {
    // This function needs to be const as it is used to create the Pics instance in a static
    // expression.
    pub const unsafe fn new(primary_offset: u8, secondary_offset: u8) -> Self {
        Pics {
            primary: Pic {
                idt_base_offset: primary_offset,
                command_port: Port::new(PRIMARY_CMD_PORT),
                data_port: Port::new(PRIMARY_DATA_PORT),
            },
            secondary: Pic {
                idt_base_offset: secondary_offset,
                command_port: Port::new(SECONDARY_CMD_PORT),
                data_port: Port::new(SECONDARY_DATA_PORT),
            },
        }
    }

    pub unsafe fn init(&mut self) {
        // Tell the two PICs we are sending the initialization sequence.
        self.primary.command_port.write(CMD_CASCADED_INIT);
        io_wait();
        self.secondary.command_port.write(CMD_CASCADED_INIT);
        io_wait();

        // Setup base offsets.
        self.primary.data_port.write(self.primary.idt_base_offset);
        io_wait();
        self.secondary
            .data_port
            .write(self.secondary.idt_base_offset);
        io_wait();

        // Cascade PICs
        self.primary.data_port.write(0x04);
        io_wait();
        self.secondary.data_port.write(0x02);
        io_wait();

        self.primary.data_port.write(CMD_8086_MODE);
        io_wait();
        self.secondary.data_port.write(CMD_8086_MODE);
        io_wait();

        // Unmask the PICs to allow future interrupts
        self.primary.data_port.write(0);
        self.secondary.data_port.write(0);
    }

    pub unsafe fn disable(&mut self) {
        self.write_masks(CMD_DISABLE_PIC, CMD_DISABLE_PIC);
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.secondary.handles_interrupt(interrupt_id) {
                self.secondary.end_interrupt();
            }
            self.primary.end_interrupt();
        }
    }

    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.primary.handles_interrupt(interrupt_id)
            || self.secondary.handles_interrupt(interrupt_id)
    }

    pub unsafe fn read_masks(&mut self) -> [u8; 2] {
        [self.primary.read_mask(), self.secondary.read_mask()]
    }

    pub unsafe fn write_masks(&mut self, primary_mask: u8, secondary_mask: u8) {
        self.primary.write_mask(primary_mask);
        self.secondary.write_mask(secondary_mask);
    }
}

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

    fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.idt_base_offset <= interrupt_id && interrupt_id < self.idt_base_offset + 8
    }
}

unsafe fn io_wait() {
    let mut port: Port<u8> = Port::new(IO_WAIT_PORT);
    port.write(0);
}
