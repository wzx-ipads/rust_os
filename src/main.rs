#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![reexport_test_harness_main = "test_main"] // Set the entry point of test framework to test_main
#![test_runner(crate::tests::test::test_runner)]

#[macro_use]
mod console;
mod drivers;
mod panic;
mod tests;
mod interrupts;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Tour of rust begins here!");
    serial_println!("Version: {}.{}", 1, 0);


    interrupts::interrupt_init();

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