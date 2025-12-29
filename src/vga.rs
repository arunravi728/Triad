use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// The VGA buffer is accessible via memory-mapped I/O to the address 0xb8000.
static VGA_BUFFER_ADDRESS: usize = 0xb8000;

// In Rust static variables are initialized at compile-time as opposed to run-time. This means we
// can only use constant functions in static variables. It also means we cannot de-reference raw
// pointers when initializing static variables.
//
// The lazy static helps solve this problem by initializing static variables at run time when they
// are first accessed. The macro ensures that initialization is atomic, even if multiple threads
// access it simultaneously for the first time, it is initialized only once without data races.
lazy_static! {
    // The ref keyword defines a static variable that acts as a reference. It is part of the lazy
    // static macro. When WRITER ois accessed, it dereferences the internal wrapper to provide
    // access to the underlying Writer instance. This ensures the Writer is initialized exactly once
    // the first time it is accessed at runtime.
    //
    // We need interior mutability for the WRITER to modify the VGA buffer. But we need to do this
    // in a thread safe manner. We thus use a spinlock. A spinlock is a lock that causes a thread
    // trying to acquire it to simply wait in a loop ("spin") while repeatedly checking whether the
    // lock is available. Since the thread remains active but is not performing a useful task, the
    // use of such a lock is a kind of busy waiting. Spinlocks are efficient if threads are likely
    // to be blocked for only short periods.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        vga_buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut Buffer) },
    });
}

// Prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Color {
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
    vga_buffer: &'static mut Buffer,
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

        self.vga_buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(Character {
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
                let character = self.vga_buffer.chars[row][column].read();
                self.vga_buffer.chars[row - 1][column].write(character);
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
            self.vga_buffer.chars[row][column].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, message: &str) -> fmt::Result {
        self.write(message);
        Ok(())
    }
}

#[test_case]
fn test_println() {
    println!("Testing println!");
}

#[test_case]
fn test_vga_buffer_height() {
    for num in 0..210 {
        println!("Printing statement number: {}", num);
    }
}

#[test_case]
fn test_vga_buffer_column_wrap_around() {
    let mut buffer = [0u8; 150];
    buffer.fill(b'A');
    let s = core::str::from_utf8(&buffer).unwrap();
    println!("{}", s);
}

#[test_case]
fn test_vga_buffer_output() {
    let test_string = "This is a test string";
    println!("{}", test_string);

    for (i, character) in test_string.chars().enumerate() {
        // We use BUFFER_HEIGHT - 2 as println! prints to the last available line in the VGA buffer
        // and then appends a new line.
        let vga_output: Character = WRITER.lock().vga_buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(character, char::from(vga_output.byte));
    }
}
