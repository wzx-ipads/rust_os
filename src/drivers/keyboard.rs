use x86_64::structures::idt::InterruptStackFrame;
use super::pic8259::{PICS, InterruptIndex};

pub extern "x86-interrupt" fn keyboard_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    //serial_println!("Get an keyboard interrupt");
    serial_print!("Get keyboard interrupt");

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}