// we add the #[macro_export] attribute to both macros to make them available everywhere in our crate
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print{
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
    () => ($crate::serial_print!("\n"));
}

