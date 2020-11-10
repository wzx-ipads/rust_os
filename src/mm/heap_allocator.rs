extern crate alloc;
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