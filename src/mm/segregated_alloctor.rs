use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr, ptr::NonNull};

struct ListNode {
    next: Option<&'static mut ListNode>,
}

/// size class:
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
/// Any allocation request larger than 2048B will fall back to
/// linked_list_allocator
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct SegregatedStorageAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

#[allow(dead_code)]
impl SegregatedStorageAllocator {
    /// Creates an empty SegregatedStorageAllocator
    pub const fn new() -> Self {
        SegregatedStorageAllocator {
            list_heads: [None; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        // Initialize the fallback allocator
        self.fallback_allocator.init(heap_start, heap_size);

        // Initailize each free block list
        for index in 0..BLOCK_SIZES.len() {
            // kernel heap is not big enough, choose 100 for now
            let layout =
                Layout::from_size_align(100 * BLOCK_SIZES[index], BLOCK_SIZES[index]).unwrap();
            let ptr = match self.fallback_allocator.allocate_first_fit(layout) {
                Ok(ptr) => ptr.as_ptr(),
                Err(_) => panic!("Out of memory when initializing SegregatedStorageAllocator"),
            };

            // stupid impl, optimize later.
            let mut last_node = ListNode {
                next: None,
            };
            let mut node_ptr = ptr as *mut ListNode;
            node_ptr.write(last_node);
            for _ in 0..99 {
                last_node = ListNode {
                    next: Some(&mut *node_ptr),
                };
                node_ptr = (node_ptr as usize + BLOCK_SIZES[index]) as *mut ListNode;
                node_ptr.write(last_node);
            }
            self.list_heads[index] = Some(&mut *node_ptr);
        }
    }

    /// Allocates using the fallback allocator.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

unsafe impl GlobalAlloc for Locked<SegregatedStorageAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            // smaller than or equals 2048
            Some(index) => {
                match allocator.list_heads[index].take() {
                    Some(list_node) => {
                        // return the first free block in the list
                        allocator.list_heads[index] = list_node.next.take();
                        list_node as *mut ListNode as *mut u8
                    }
                    None => {
                        // no free block in this list
                        // alloc from fallback allocator
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            // larger than 2048B
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let list_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };

                // verify that block has size and alignment required for storing node
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

                let node_ptr = ptr as *mut ListNode;
                node_ptr.write(list_node);
                allocator.list_heads[index] = Some(&mut *node_ptr);
            }
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}
