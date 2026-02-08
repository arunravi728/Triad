use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb888, RgbColor},
    Pixel,
};

pub struct Draw<'f> {
    framebuffer: &'f mut FrameBuffer,
}

impl<'f> Draw<'f> {
    pub fn new(framebuffer: &'f mut FrameBuffer) -> Self {
        Draw { framebuffer }
    }

    fn draw_pixel(&mut self, Pixel(position, color): Pixel<Rgb888>) {
        let width = self.framebuffer.info().width;
        let height = self.framebuffer.info().height;

        let (x, y) = {
            let c: (i32, i32) = position.into();
            (c.0 as usize, c.1 as usize)
        };

        if (0..width).contains(&x) && (0..height).contains(&y) {
            let color = Color {
                red: color.r(),
                green: color.g(),
                blue: color.b(),
            };
            set_pixel(self.framebuffer, Position { x, y }, color);
        }
    }
}

impl<'f> DrawTarget for Draw<'f> {
    type Color = Rgb888;

    /// Drawing operations can never fail.
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels.into_iter() {
            self.draw_pixel(pixel);
        }

        Ok(())
    }
}

impl<'f> OriginDimensions for Draw<'f> {
    fn size(&self) -> Size {
        let info = self.framebuffer.info();

        Size::new(info.width as u32, info.height as u32)
    }
}

pub fn set_pixel(framebuffer: &mut FrameBuffer, position: Position, color: Color) {
    let info: FrameBufferInfo = framebuffer.info();

    let offset = {
        let line_offset = position.y * info.stride;
        let pixel_offset = line_offset + position.x;
        pixel_offset * info.bytes_per_pixel
    };

    let buffer = &mut framebuffer.buffer_mut()[offset..];

    match info.pixel_format {
        PixelFormat::Rgb => {
            buffer[0] = color.red;
            buffer[1] = color.green;
            buffer[2] = color.blue;
        }
        PixelFormat::Bgr => {
            buffer[0] = color.blue;
            buffer[1] = color.green;
            buffer[2] = color.red;
        }
        PixelFormat::U8 => {
            buffer[0] = (color.red + color.green + color.blue) / 3;
        }
        other => panic!("Unknown pixel format: {other:?}"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
