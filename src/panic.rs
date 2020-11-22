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

// The Rust compiler does not permit trait implementations for types defined in other crates:
// unsafe impl GlobalAlloc for spin::Mutex<BumpAllocator> {...}  Wrong!
// So, use a warpper Lock<T> to permit trait implementation
pub struct Locked<T> {
    inner: spin::Mutex<T>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    // Mutex::<A>::lock() returns MutexGuard<A>
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}
