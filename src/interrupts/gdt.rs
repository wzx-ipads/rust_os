/*
 * gdt(Global Descriptor Table)
 * The GDT is a structure that contains the segments of the program. It was used on
 * older architectures to isolate programs from each other, before paging became the
 * standard. While segmentation is no longer supported in 64-bit mode, the GDT still
 * exists. It is mostly used for two things: Switching between kernel space and user
 * space, and loading a TSS structure.
 */
use lazy_static::lazy_static;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

// pub struct TaskStateSegment {
//     reserved_1: u32,
//     /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
//     pub privilege_stack_table: [VirtAddr; 3],
//     reserved_2: u64,
//     /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
//     pub interrupt_stack_table: [VirtAddr; 7],
//     reserved_3: u64,
//     reserved_4: u16,
//     /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base.
//     pub iomap_base: u16,
// }

// Use the 0th entry in IST as the double fault stack
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; //20k
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

// pub struct GlobalDescriptorTable {
//     table: [u64; 8],
//     next_free: usize,
// }

struct Selector {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selector) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selector {
                code_selector,
                tss_selector,
            },
        )
    };
}

// Initialize GDT and reload the cs segment register and load our TS
pub fn gdt_init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;
    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
