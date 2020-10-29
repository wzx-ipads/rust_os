#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"] // Set the entry point of test framework to test_main
#![test_runner(crate::tests::test::test_runner)]

#[macro_use]
mod console;
mod drivers;
mod panic;
mod tests;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("Tour of rust begins here!");
    serial_println!("Version: {}.{}", 1, 0);

    #[cfg(test)]
    test_main();

    loop {}
}
