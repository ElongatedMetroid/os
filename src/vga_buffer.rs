use volatile::Volatile;
use core::fmt::{self, Write};

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
    pub fn print_something() {
        let mut writer = Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            // Cast the integer 0xb8000 as a mutable raw pointer
            // then convert it to a mutable reference by 
            // dereferencing it and borrowing it again with &mut
            // (unsafe is required since the compiler cant 
            // guarantee that the rwo pointer is valid)
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        };
    
        writer.write_byte(b'H');
        writer.write_string("ello ");
        
        // Since we implemented Write for Writer we can use 
        // Rust's built-in write! and writeln! formatting macros
        write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not part of the printable ASCII range,
                // write a square character
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
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

    fn new_line(&mut self) {  }
}