use bootloader_api::info::FrameBufferInfo;
use bootloader_x86_64_common::{framebuffer::FrameBufferWriter, serial::SerialPort};
use conquer_once::spin::OnceCell;
use core::fmt::Write;
use spinning_top::Spinlock;

// The global logger instance used for the `log` crate.
pub static LOGGER: OnceCell<KernelLogger> = OnceCell::uninit();

// A logger instance protected by a spinlock.
pub struct KernelLogger {
    framebuffer: Option<Spinlock<FrameBufferWriter>>,
    serial: Option<Spinlock<SerialPort>>,
}

impl KernelLogger {
    pub fn new(
        framebuffer: &'static mut [u8],
        info: FrameBufferInfo,
        frame_buffer_logger_status: bool,
        serial_logger_status: bool,
    ) -> Self {
        let framebuffer = match frame_buffer_logger_status {
            true => Some(Spinlock::new(FrameBufferWriter::new(framebuffer, info))),
            false => None,
        };

        let serial = match serial_logger_status {
            true => Some(Spinlock::new(unsafe { SerialPort::init() })),
            false => None,
        };

        KernelLogger {
            framebuffer,
            serial,
        }
    }

    // Force-unlocks the logger to prevent a deadlock.
    //
    // ## Safety
    // This method is not memory safe and should be only used when absolutely necessary.
    pub unsafe fn force_unlock(&self) {
        if let Some(framebuffer) = &self.framebuffer {
            unsafe { framebuffer.force_unlock() };
        }
        if let Some(serial) = &self.serial {
            unsafe { serial.force_unlock() };
        }
    }
}

impl log::Log for KernelLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if let Some(framebuffer) = &self.framebuffer {
            let mut framebuffer = framebuffer.lock();
            writeln!(framebuffer, "{:5}: {}", record.level(), record.args()).unwrap();
        }
        if let Some(serial) = &self.serial {
            let mut serial = serial.lock();
            writeln!(serial, "{:5}: {}", record.level(), record.args()).unwrap();
        }
    }

    fn flush(&self) {}
}

pub fn init_logger(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let logger = LOGGER.get_or_init(move || KernelLogger::new(buffer, info, true, false));
    log::set_logger(logger).expect("Logger already set");
    log::set_max_level(log::LevelFilter::Trace);
    log::info!("Hello, Kernel Mode!");
}
