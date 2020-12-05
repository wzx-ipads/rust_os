use super::buddy_allocator::BuddyAllocator;

const MAX_SLAB_ORDER: usize = 10;
const MIN_SLAB_ORDER: usize = 5;
const DEFAULT_SLAB_SIZE: usize = 8 * 1024;

#[repr(C)]
// 24 bytes in total
pub struct slab_header {
    free_list_head: Option<&'static mut slab_slot_list>,
    next_slab: Option<&'static mut slab_header>,
    order: u32,
}

#[repr(C)]
struct slab_slot_list {
    next_free: Option<&'static mut slab_slot_list>,
}

/* Use buddy system as the fallback allocator */
struct SlabAllocator {
    slabs: [Option<&'static mut slab_header>; MAX_SLAB_ORDER],
    fallback_allocator: BuddyAllocator,
}

impl slab_header {
    pub fn new() -> Self {
        slab_header {
            free_list_head: None,
            next_slab: None,
            order: 0,
        }
    }
}

// if size <= PAGE_SIZE {
//     return 0;
// }
// let mut order = 0;
// let page_num = round_up(size, PAGE_SIZE) / PAGE_SIZE;
// let mut tmp = page_num;

// while tmp > 1 {
//     tmp >>= 1;
//     order += 1;
// }

// if page_num > (1 << order) {
//     order += 1;
// }

// order

pub fn size_to_order(size: usize) -> u32 {
    let mut order = 0;
    let mut tmp = size;

    while tmp > 1 {
        tmp >>= 1;
        order += 1;
    }

    if size > (1 << order) {
        order += 1;
    }
    if order < MIN_SLAB_ORDER {
        order = MIN_SLAB_ORDER;
    }
    order as u32
}

impl SlabAllocator {
    pub fn new() -> Self {
        SlabAllocator {
            slabs: [None; MAX_SLAB_ORDER],
            fallback_allocator: BuddyAllocator::new(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        /* buddy system should be initialized first */
        self.fallback_allocator.init(heap_start, heap_size);

        for order in MIN_SLAB_ORDER..MAX_SLAB_ORDER {
            let page = self
                .fallback_allocator
                .get_free_pages(DEFAULT_SLAB_SIZE, DEFAULT_SLAB_SIZE)
                .unwrap();
            let slab_start_addr = self.fallback_allocator.page_to_virt(page);
            //let slot_num = DEFAULT_SLAB_SIZE /
        }
    }
}
