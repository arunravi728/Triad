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

// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::print::serial::_serial_print(format_args!($($arg)*));
    };
}

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL.lock().write_fmt(args).expect("Printing to serial failed");
}

#[test_case]
fn test_serial_println() {
    serial_println!("Testing serial_println!");
}

#[test_case]
fn test_serial_println_many() {
    for num in 0..10 {
        serial_println!("Printing statement number: {}", num);
    }
}
