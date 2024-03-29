use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    /// Global interface to the writer
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// print! and println! macros copied, but changed to use our own
// _print function.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
// Since the macros need to call _print from outside this module
// the function needs to be public. However since we consider 
// this a private implementation detail, we used the doc(hidden)
// attributed to hide it from the generated documentation
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    
    // Execute the closure with interupts disabled
    interrupts::without_interrupts(|| {
        // This should not panic since Ok(()) is always returned 
        // from write_str
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Each enum variant is stored as a u8
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
// Assure ColorCode has the exact same data layout as a u8
#[repr(transparent)]
/// Contains the full color byte
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Guarantees that the struct fields are laid out exactaly like
// a C struct, guaranteing the correct field ordering
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    /// 2D array of ScreenChar's
    // Marked as volitile since we only write to the Buffer and
    // never read from it again. The compiler doesn’t know that 
    // we really access VGA buffer memory (instead of normal RAM) 
    // and knows nothing about the side effect that some 
    // characters appear on the screen. So it might decide that 
    // these writes are unnecessary and can be omitted. 
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

// Implement Write for Writer so we can use Rust's formating 
// macros
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Writer {
    /// Write a string to the VGA Buffer
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII byte or newline
                0x20..=0x7e | b'\n' | b'\t' => self.write_byte(byte),
                // Not part of the printable ASCII range,
                // write a square character
                _ => self.write_byte(0xfe),
            }
        }
    }
    /// Write a single byte to the VGA Buffer
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\t' => {
                for _ in 0..5 {
                    self.write_byte(b' ');
                }
            }
            byte => {
                // If we are at the end of the column...
                if self.column_position >= BUFFER_WIDTH {
                    // Add a new line
                    self.new_line();
                }

                // Set row to the last row
                let row = BUFFER_HEIGHT - 1;
                // Set col to the current column_position
                let col = self.column_position;

                // Set the color_code to the current color_code
                let color_code = self.color_code;
                // Write the the row and col of the VGA buffer
                self.buffer.chars[row][col].write(ScreenChar {
                    // Set the character to the byte passed in
                    ascii_character: byte,
                    // Set the color_code to the current color
                    // code being used
                    color_code,
                });
                // Increment the column_position
                self.column_position += 1;
            }
        }
    }

    /// Move every character one line up (the top line will be 
    /// deleted) and start at the beginning of the last line 
    /// again
    fn new_line(&mut self) { 
        // Omit 0th row since its the row that is shifted off
        // screen.
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        // Clear last row
        self.clear_row(BUFFER_HEIGHT - 1);
        // Set the column position to zero
        // |
        // |Hello World\n <-- Column Postion 0
        // |<-- Column Postion = 0
        self.column_position = 0;
    }
    /// Clear the specified row with spaces
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        // Could fail because of the timer interrupt handler
        // running between println and the reading of screen
        // chars, ex.
        // Error: panicked at 'assertion failed: `(left == right)`
        // left: `'.'`,
        // right: `'S'`', src/vga_buffer.rs:205:9
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}