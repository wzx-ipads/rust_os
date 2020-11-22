extern crate alloc;
use super::{bump_allocator::BumpAllocator, pool_allocator::LinkedListAllocator, Locked};
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

pub const HEAP_START: usize = 0x444444440000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100K

#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

pub fn init_kernel_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let heap_pages = HEAP_SIZE / (4 * 1024);
    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(HEAP_START as u64));

    for i in 0..heap_pages {
        let curr_page = page + (i * 4 * 1024) as u64;
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::<Size4KiB>::FrameAllocationFailed)?;
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
