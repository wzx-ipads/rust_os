pub mod page_table;
pub mod allocator;
pub mod heap_allocator;
pub mod bump_allocator;
pub mod pool_allocator;
pub mod segregated_alloctor;

fn round_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // addr already aligned
    } else {
        addr - remainder + align
    }
}

fn round_down(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr
    } else {
        addr - remainder
    }
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