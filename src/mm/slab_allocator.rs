use super::buddy_allocator::{BuddyAllocator, PAGE_SIZE};
use super::{round_down, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

const MAX_SLAB_ORDER: usize = 12;
const MIN_SLAB_ORDER: usize = 5;
const DEFAULT_SLAB_SIZE: usize = 8 * 1024;

#[repr(C)]
// 24 bytes in total
pub struct SlabHeader {
    free_list_head: Option<&'static mut SlotListNode>,
    next_slab: Option<&'static mut SlabHeader>,
    order: u32,
}

#[repr(C)]
struct SlotListNode {
    next_free: Option<&'static mut SlotListNode>,
}

/* Use buddy system as the fallback allocator */
pub struct SlabAllocator {
    slabs: [Option<&'static mut SlabHeader>; MAX_SLAB_ORDER],
    fallback_allocator: BuddyAllocator,
}

impl SlabHeader {
    pub fn new() -> Self {
        SlabHeader {
            free_list_head: None,
            next_slab: None,
            order: 0,
        }
    }
}

impl SlotListNode {
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
}

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
    pub const fn new() -> Self {
        SlabAllocator {
            slabs: [None; MAX_SLAB_ORDER],
            fallback_allocator: BuddyAllocator::new(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        /* buddy system should be initialized first */
        self.fallback_allocator.init(heap_start, heap_size);

        for order in MIN_SLAB_ORDER..MAX_SLAB_ORDER {
            self.slabs[order] = self.init_slab_cache(order as u32);
        }
    }

    pub unsafe fn init_slab_cache(&mut self, order: u32) -> Option<&'static mut SlabHeader> {
        let page = self
            .fallback_allocator
            .get_free_pages(DEFAULT_SLAB_SIZE, DEFAULT_SLAB_SIZE)
            .unwrap();
        let slot_size = 1 << order;
        let slab_start_addr = self.fallback_allocator.page_to_virt(page);

        let slot_num = DEFAULT_SLAB_SIZE / (slot_size) - 1;
        let mut slot_addr = slab_start_addr + slot_num * slot_size;
        let mut next_slot: Option<&'static mut SlotListNode> = None;
        while slot_addr > slab_start_addr {
            let slot_ptr = slot_addr as *mut SlotListNode;
            let slot = SlotListNode {
                next_free: next_slot.take(),
            };
            slot_ptr.write(slot);
            next_slot = Some(&mut *slot_ptr);
            slot_addr -= slot_size;
        }

        let slab_header_ptr = slab_start_addr as *mut SlabHeader;
        let slab_header = SlabHeader {
            free_list_head: next_slot.take(),
            next_slab: None,
            order: order as u32,
        };
        slab_header_ptr.write(slab_header);

        let page_num = DEFAULT_SLAB_SIZE / PAGE_SIZE;
        for i in 0..page_num {
            let slab_page = self
                .fallback_allocator
                .virt_to_page(slab_start_addr + i * PAGE_SIZE);
            slab_page.slab = Some(&mut *slab_header_ptr);
        }
        Some(&mut *slab_header_ptr)
    }

    unsafe fn find_free_slot(&mut self, order: u32) -> Option<&'static mut SlotListNode> {
        let slab_header = self.slabs[order as usize].as_mut().unwrap();
        match slab_header.free_list_head.take() {
            Some(free_slot) => {
                slab_header.free_list_head = free_slot.next_free.take();
                return Some(free_slot);
            }
            None => {
                let mut tmp_header = slab_header;
                while let Some(ref mut header) = tmp_header.next_slab {
                    match header.free_list_head.take() {
                        Some(free_slot) => {
                            header.free_list_head = free_slot.next_free.take();
                            return Some(free_slot);
                        }
                        None => {}
                    }
                    tmp_header = tmp_header.next_slab.as_mut().unwrap();
                }

                // No free slot in current slab list. Get a new slab from buddy
                let new_slab_header = self.init_slab_cache(order).unwrap();
                let free_slot = new_slab_header.free_list_head.take().unwrap();
                new_slab_header.free_list_head = free_slot.next_free.take();
                new_slab_header.next_slab = self.slabs[order as usize].take();
                self.slabs[order as usize] = Some(new_slab_header);
                return Some(free_slot);
            }
        }
    }
}

unsafe impl GlobalAlloc for Locked<SlabAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let order = size_to_order(layout.size());
        if order >= MAX_SLAB_ORDER as u32 {
            let allocator = &mut self.lock().fallback_allocator;
            match allocator.get_free_pages(layout.size(), layout.align()) {
                Some(page) => allocator.page_to_virt(page) as *mut u8,
                None => ptr::null_mut(),
            }
        } else {
            let mut allocator = self.lock();
            let free_slot = allocator.find_free_slot(order).unwrap();
            return free_slot.start_addr() as *mut u8;
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        let page_addr = round_down(ptr as usize, PAGE_SIZE);
        let page = allocator.fallback_allocator.virt_to_page(page_addr);
        match page.slab.as_mut() {
            Some(slab_header) => {
                /* Free this slot in the corresponding slab */
                let slot_ptr = ptr as *mut SlotListNode;
                let free_slot = SlotListNode {
                    next_free: slab_header.free_list_head.take(),
                };
                slot_ptr.write(free_slot);
                /* Insert this slot to the free list */
                slab_header.free_list_head = Some(&mut *slot_ptr);
            }
            None => {
                /* Free the page in buddy system */
                allocator.fallback_allocator.free_pages(page);
            }
        }
    }
}
