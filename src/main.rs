#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![reexport_test_harness_main = "test_main"] // Set the entry point of test framework to test_main
#![test_runner(crate::tests::test::test_runner)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

#[macro_use]
mod console;
mod drivers;
mod interrupts;
mod mm;
mod panic;
mod tests;

extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use bootloader::{entry_point, BootInfo};
use mm::{allocator, heap_allocator, page_table};
use x86_64::VirtAddr;
entry_point!(kernel_main);

/// This is a normal Rust function. Bootloader will call this
/// function as the entry point of the kernel.
pub fn kernel_main(bootinfo: &'static BootInfo) -> ! {
    println!("Tour of rust begins here!");
    serial_println!("Version: {}.{}", 1, 0);

    interrupts::interrupt_init();

    let phys_mem_offset = VirtAddr::new(bootinfo.physical_memory_offset);
    let mut mapper = unsafe { page_table::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { allocator::BootInfoFrameAllocator::init(&bootinfo.memory_map) };
    heap_allocator::init_kernel_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let x = Box::new(0x41);
    println!("{:p}", x);

     // create a reference counted vector -> will be freed when count reaches 0
     let reference_counted = Rc::new(vec![1, 2, 3]);
     let cloned_reference = reference_counted.clone();
     println!("current reference count is {}", Rc::strong_count(&cloned_reference));
     core::mem::drop(reference_counted);
     println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3(); // new
    serial_println!("It did not crash!");

    #[cfg(test)]
    test_main();

    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
