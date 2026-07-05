use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::{fmt, ptr};
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};

use crate::print::framebuffer::{Color, Position};

// Additional vertical space between lines
const LINE_SPACING: usize = 2;

// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

// Backup character if a desired symbol is not available by the font.
pub const BACKUP_CHAR: char = '�';

// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size20;

// The width of each single symbol of the mono space font.
pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;

// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

pub struct Writer {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    pos: Position,
    color: Color,
}

impl Writer {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            pos: Position { x: 0, y: 0 },
            color: Color {
                red: 0,
                green: 255,
                blue: 0,
            },
        };
        logger.clear();
        logger
    }

    fn newline(&mut self) {
        self.pos.y += CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.pos.x = BORDER_PADDING;
    }

    pub fn clear(&mut self) {
        self.pos.x = BORDER_PADDING;
        self.pos.y = BORDER_PADDING;
        self.framebuffer.fill(0);
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.pos.x + CHAR_RASTER_WIDTH;
                if new_xpos >= self.width() {
                    self.newline();
                }
                let new_ypos = self.pos.y + CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }
                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.pos.x + x, self.pos.y + y, *byte);
            }
        }
        self.pos.x += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [
                (self.color.red as u16 * intensity as u16 / 255) as u8,
                (self.color.green as u16 * intensity as u16 / 255) as u8,
                (self.color.blue as u16 * intensity as u16 / 255) as u8,
                0,
            ],
            PixelFormat::Bgr => [
                (self.color.blue as u16 * intensity as u16 / 255) as u8,
                (self.color.green as u16 * intensity as u16 / 255) as u8,
                (self.color.red as u16 * intensity as u16 / 255) as u8,
                0,
            ],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }
}

unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
