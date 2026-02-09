use bootloader_api::info::FrameBufferInfo;
use bootloader_x86_64_common::framebuffer::FrameBufferWriter;
use conquer_once::spin::OnceCell;
use core::fmt::Write;
use spinning_top::Spinlock;

// The global logger instance used for the `log` crate.
pub static LOGGER: OnceCell<KernelLogger> = OnceCell::uninit();

// A logger instance protected by a spinlock.
pub struct KernelLogger {
    framebuffer: Option<Spinlock<FrameBufferWriter>>,
}

impl KernelLogger {
    pub fn new(
        framebuffer: &'static mut [u8],
        info: FrameBufferInfo,
        frame_buffer_logger_status: bool,
    ) -> Self {
        let framebuffer = match frame_buffer_logger_status {
            true => Some(Spinlock::new(FrameBufferWriter::new(framebuffer, info))),
            false => None,
        };

        KernelLogger { framebuffer }
    }

    pub fn print_raw(&self, args: core::fmt::Arguments) {
        if let Some(framebuffer) = &self.framebuffer {
            let mut framebuffer = framebuffer.lock();
            framebuffer.write_fmt(args).unwrap();
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
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::print::log::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::print::log::kprint!("\n"));
    ($($arg:tt)*) => ($crate::print::log::kprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    if let Some(logger) = LOGGER.get() {
        logger.print_raw(args);
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
    }

    fn flush(&self) {}
}

pub fn init_logger(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let logger = LOGGER.get_or_init(move || KernelLogger::new(buffer, info, true));
    log::set_logger(logger).expect("Logger already set");
    log::set_max_level(log::LevelFilter::Trace);
}
