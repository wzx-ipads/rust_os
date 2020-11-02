use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use core::fmt;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe {
            SerialPort::new(0x3f8) // 0x3f8 is the standard port number for the first serial interface.
        };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    /*
     * Some interrupt handlers will also use SERIAL1, so disable
     * interrupt to avoid deadlock.
     */
    interrupts::without_interrupts( || {
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
    });
}
