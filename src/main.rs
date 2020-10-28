#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"] // Set the entry point of test framework to test_main

mod drivers;
use crate::drivers::vga_buffer;
use core::panic::PanicInfo;

// Only for tests
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Tour of rust begins here!");
    println!("Version: {}.{}", 1, 0);

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

#[test_case]
fn trivial_assertion() {
    print!("assertion test...");
    assert_eq!(1,1);
    print!("passed");
}