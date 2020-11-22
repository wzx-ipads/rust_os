extern crate alloc;
use super::{Locked, segregated_alloctor::SegregatedStorageAllocator};
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

pub const HEAP_START: usize = 0x444444440000;
pub const HEAP_SIZE: usize = 1 * 1024 * 1024; // 1M

// Use SegregatedStorageAllocator as the default heap allocator
#[global_allocator]
static ALLOCATOR: Locked<SegregatedStorageAllocator> = Locked::new(SegregatedStorageAllocator::new());

pub fn init_kernel_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let heap_pages = HEAP_SIZE / (4 * 1024);
    let start_va = VirtAddr::new(HEAP_START as u64);
    let mut curr_page;

    for i in 0..heap_pages {
        let va = start_va + (i * 4 * 1024) as u64;
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::<Size4KiB>::FrameAllocationFailed)?;
        curr_page = Page::containing_address(va);
        unsafe {
            mapper
                .map_to(
                    curr_page,
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    frame_allocator,
                )?
                .flush();
        }
    }

    unsafe {
        /*
         * We init ALLOCATOR after the heap mapping because the
         * init function already tries to write to the heap memory
         */
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    Ok(())
}

#[alloc_error_handler]
pub fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}

/*
 * The alloc crate in rust standard library has some additional requirements.
 * The #[global_allocator] attribute must be applied to a static variable that
 * implements the GlobalAlloc trait as following. Memory allocation can fail so
 * a function with attribute #[alloc_error_handler] is also needed.
 */
// pub unsafe trait GlobalAlloc {
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8;
//     unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);

//     unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 { ... }
//     unsafe fn realloc(
//         &self,
//         ptr: *mut u8,
//         layout: Layout,
//         new_size: usize
//     ) -> *mut u8 { ... }
// }

#[test_case]
fn simple_allocation() {
    use alloc::{boxed::Box};
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    use alloc::{vec::Vec};
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    use alloc::{boxed::Box};
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}