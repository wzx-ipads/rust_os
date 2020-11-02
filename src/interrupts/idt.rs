use lazy_static::lazy_static;
use super::gdt;
use super::timer;
use crate::drivers::pic8259;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
// InterruptDescriptorTable are defined as following

// pub struct InterruptDescriptorTable {
//     pub divide_by_zero: Entry<HandlerFunc>,
//     pub debug: Entry<HandlerFunc>,
//     pub non_maskable_interrupt: Entry<HandlerFunc>,
//     pub breakpoint: Entry<Handle0rFunc>,
//     pub overflow: Entry<HandlerFunc>,
//     pub bound_range_exceeded: Entry<HandlerFunc>,
//     pub invalid_opcode: Entry<HandlerFunc>,
//     pub device_not_available: Entry<HandlerFunc>,
//     pub double_fault: Entry<HandlerFuncWithErrCode>,
//     pub invalid_tss: Entry<HandlerFuncWithErrCode>,
//     pub segment_not_present: Entry<HandlerFuncWithErrCode>,
//     pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,
//     pub general_protection_fault: Entry<HandlerFuncWithErrCode>,
//     pub page_fault: Entry<PageFaultHandlerFunc>,
//     pub x87_floating_point: Entry<HandlerFunc>,
//     pub alignment_check: Entry<HandlerFuncWithErrCode>,
//     pub machine_check: Entry<HandlerFunc>,
//     pub simd_floating_point: Entry<HandlerFunc>,
//     pub virtualization: Entry<HandlerFunc>,
//     pub security_exception: Entry<HandlerFuncWithErrCode>,
//     // some fields omitted
// }

// Create a new Interrupt descriptor table
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        /* 
         * The InterruptDescriptorTable struct implements the IndexMut trait, 
         * so we can access individual entries through array indexing syntax.
         */
        idt[pic8259::InterruptIndex::Timer.as_usize()].set_handler_fn(timer::timer_interrupt_handler) ;
        idt
    };
}

pub fn idt_init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    // the error_code in double fault is always 0
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
