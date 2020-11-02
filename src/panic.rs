use core::panic::PanicInfo;
use crate::tests::test;
use crate::hlt_loop;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("{}", _info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("test failed!");
    serial_println!("{}", _info);
    test::exit_qemu(test::QemuExitCode::Failed);
    loop {};
}
