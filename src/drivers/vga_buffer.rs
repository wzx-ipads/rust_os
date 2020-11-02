use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// Using static mut is highly discouraged
lazy_static! {
    /// A global writer which can write to VGA text buffer
    pub static ref VGA_WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[allow(dead_code)]
// By deriving the Copy, Clone, Debug, PartialEq, and Eq traits, we enable copy semantics for the type and make it printable and comparable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // Each enum variant is stored as an u8
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
// To ensure that the ColorCode has the exact same data layout as an u8, we use the repr(transparent) attribute.
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // It guarantees that the struct's fields are laid out exactly like in a C struct and thus guarantees the correct field ordering.
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// Use volatile to avoid compiler optimization
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    fn new_line(&mut self) {
        for row in 0..BUFFER_HEIGHT - 1 {
            for col in 0..BUFFER_WIDTH {
                let temp_char = self.buffer.chars[row + 1][col].read();
                self.buffer.chars[row][col].write(temp_char);
            }
        }
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][col].write(ScreenChar {
                ascii_character: 0,
                color_code: ColorCode::new(Color::Black, Color::Black),
            });
        }
        self.column_position = 0;
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let schar = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.buffer.chars[row][col].write(schar);
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/*
 * Since the macros need to be able to call _print from outside of the module,
 * the function needs to be public. However, since we consider this a private
 * implementation detail, we add the doc(hidden) attribute to hide it from the
 * generated documentation.
 */
 #[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    /*
     * Some interrupt handlers will also use VGA_WRITER, so disable
     * interrupt to avoid deadlock.
     */
    interrupts::without_interrupts( || {
        VGA_WRITER.lock().write_fmt(args).unwrap();
    });
}
