use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;
use crate::hlt_loop;

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: PageFaultErrorCode) {
    let fault_addr: u64 = unsafe {
        Cr2::read().as_u64()
    };
    serial_println!("Page fault at {:#x}", fault_addr);
    serial_println!("Error Code: {:?}", _error_code);
    serial_println!("{:#?}", stack_frame);
    hlt_loop();
}