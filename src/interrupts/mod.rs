mod idt;
mod gdt;
mod timer;

use crate::drivers::pic8259;

pub fn interrupt_init() {
    gdt::gdt_init();
    idt::idt_init();
}