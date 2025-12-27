use volatile::Volatile;

// The VGA buffer is accessible via memory-mapped I/O to the address 0xb8000.
static VGA_BUFFER_ADDRESS: usize = 0xb8000;

pub fn print(message: &str) {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut Buffer) },
    };

    writer.write(message);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

// In the VGA buffer -
// 1. The first four bits define the foreground color
// 2. The next three bits define the background color
// 3. The last bit defines whether the character should blink.
impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// Each VGA buffer character contains an ASCII and a color byte.
//
// Field ordering in default structs is undefined in Rust. repr(C) attribute guarantees that the
// struct’s fields are laid out exactly as a C struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Character {
    byte: u8,
    color_code: ColorCode,
}

// The VGA text buffer is a two-dimensional array with typically 25 rows and 80 columns, which is
// directly rendered to the screen.
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    // Volatile avoids any future Rust compiler optimizations that might assume the write to the
    // buffer is not necessary.
    chars: [[Volatile<Character>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// A public writer interface used to output text to the VGA buffer.
pub struct Writer {
    // Keeps track of the current position in the last row
    column_position: usize,
    color_code: ColorCode,
    // Reference to the VGA buffer. We use a static lifetime here to tell the compiler that the VGA
    // buffer reference for the lifetime of the program.
    buffer: &'static mut Buffer,
}

const NEW_LINE_CHARACTER: u8 = b'\n';

impl Writer {
    pub fn write(&mut self, message: &str) {
        for byte in message.bytes() {
            match byte {
                NEW_LINE_CHARACTER => self.new_line(),
                byte => self.write_byte(byte),
            }
        }
    }

    fn write_byte(&mut self, byte: u8) {
        if self.column_position >= BUFFER_WIDTH {
            self.new_line();
        }

        self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(Character {
            byte: byte,
            color_code: self.color_code,
        });

        self.column_position += 1;
    }

    // If we encounter a new line, we move all the characters up one row.
    // If the number of rows exceeds the buffer's height, we lose the oldest text.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for column in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][column].read();
                self.buffer.chars[row - 1][column].write(character);
            }
        }

        self.column_position = 0;
        self.clear_row(BUFFER_HEIGHT - 1);
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Character {
            byte: b' ',
            color_code: self.color_code,
        };

        for column in 0..BUFFER_WIDTH {
            self.buffer.chars[row][column].write(blank);
        }
    }
}
