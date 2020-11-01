pub mod idt;

pub fn interrupt_init() {
    idt::init_idt();
}