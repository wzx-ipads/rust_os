mod idt;
mod gdt;
mod timer;
mod page_fault;

use crate::drivers::pic8259;

pub fn interrupt_init() {
    gdt::gdt_init();
    idt::idt_init();
    unsafe {
        pic8259::PICS.lock().initialize();
    }
    // execute sti instruction ("set interrupts") to enable external interrupts
    x86_64::instructions::interrupts::enable();
}