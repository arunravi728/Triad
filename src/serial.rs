use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    // UART uses memory-mapped I/O. SerialPort::new expects the address of the first I/O port of the
    // UART interface. It uses this to calculate the address of all other ports.
    pub static ref SERIAL: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}