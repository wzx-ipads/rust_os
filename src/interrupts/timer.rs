use x86_64::structures::idt::InterruptStackFrame;
use crate::drivers::pic8259::{PICS, InterruptIndex};

pub extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    //serial_print!(".");

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}