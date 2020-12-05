use super::{round_down, round_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};
use super::slab_allocator::slab_header;

const MAX_BUDDY_ORDER: usize = 8;
const PAGE_SIZE: usize = 1 << 12;
const BUDDY_PAGE_ORDER: usize = 12;

#[repr(C)]
pub struct page {
    prev: Option<&'static mut page>,
    next: Option<&'static mut page>,
    order: u32,
    allocated: bool,
    slab: Option<&'static mut slab_header>,
}

pub struct BuddyAllocator {
    list_heads: [Option<&'static mut page>; MAX_BUDDY_ORDER],
    page_num: u64,
    // make sure the start address is 4k aligned
    start_addr: usize,
    heap_start: usize,
    heap_size: usize,
}

impl page {
    pub fn new() -> Self {
        page {
            prev: None,
            next: None,
            order: 0,
            allocated: false,
            slab: None,
        }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
}

fn size_to_page_order(size: usize) -> u32 {
    if size <= PAGE_SIZE {
        return 0;
    }
    let mut order = 0;
    let page_num = round_up(size, PAGE_SIZE) / PAGE_SIZE;
    let mut tmp = page_num;

    while tmp > 1 {
        tmp >>= 1;
        order += 1;
    }

    if page_num > (1 << order) {
        order += 1;
    }

    order
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        BuddyAllocator {
            list_heads: [None; MAX_BUDDY_ORDER],
            page_num: 0,
            start_addr: 0,
            heap_start: 0,
            heap_size: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        let page_num = heap_size / (PAGE_SIZE + mem::size_of::<page>());
        self.page_num = page_num as u64;
        self.heap_start = heap_start;
        self.heap_size = heap_size;
        self.start_addr = round_up(heap_start + page_num * mem::size_of::<page>(), PAGE_SIZE);

        /* Init the page_metadata area. */
        for index in 0..page_num {
            let page_ptr = (heap_start + index * mem::size_of::<page>()) as *mut page;
            let curr_page = page {
                prev: None,
                next: None,
                order: 0,
                allocated: true,
                slab: None,
            };
            page_ptr.write(curr_page);
        }

        /* Put each physical memory page into the free lists. */
        for index in 0..page_num {
            let page_ptr = (heap_start + index * mem::size_of::<page>()) as *mut page;
            self.free_pages(&mut *page_ptr);
        }
    }

    pub fn page_to_virt(&self, page: &'static page) -> usize {
        let index = (page.start_addr() - self.heap_start) / mem::size_of::<page>();
        assert!((index as i32) >= 0 && (index as u64) < self.page_num);
        self.start_addr + PAGE_SIZE * index
    }

    pub unsafe fn virt_to_page(&self, virt_addr: usize) -> &'static mut page {
        assert!(virt_addr >= self.heap_start && virt_addr < self.heap_start + self.heap_size);
        let index = (round_down(virt_addr, PAGE_SIZE) - self.start_addr) / PAGE_SIZE;
        let page_ptr = (self.heap_start + index * mem::size_of::<page>()) as *mut page;
        &mut *page_ptr
    }

    pub unsafe fn find_buddy_chunk(&self, page: &'static page) -> &'static mut page {
        let va = self.page_to_virt(page);

        let order = page.order as usize;

        let buddy_chunk_addr = ((va - self.start_addr)
            ^ ((1 as usize) << (order + BUDDY_PAGE_ORDER)))
            + self.start_addr;
        assert!(
            buddy_chunk_addr >= self.heap_start
                && buddy_chunk_addr < self.heap_start + self.heap_size
        );
        self.virt_to_page(buddy_chunk_addr)
    }

    pub unsafe fn merge_page(&mut self, page: &'static mut page) -> &'static mut page {
        if page.order >= (MAX_BUDDY_ORDER - 1) as u32 {
            return page;
        }
        let mut page_ptr = page.start_addr() as *mut page;
        let buddy_page = self.find_buddy_chunk(&mut *page_ptr);

        if buddy_page.allocated == true {
            return page;
        }

        if buddy_page.order != page.order {
            return page;
        }

        /* Remove the buddy_chunk from its current free list. */
        match buddy_page.prev.take() {
            // not the head of free list
            Some(prev_page) => {
                prev_page.next = buddy_page.next.take();
            }
            // head of the free list
            None => {
                self.list_heads[buddy_page.order as usize] = buddy_page.next.take();
            }
        }

        /* Merge the two buddies and get a larger chunk @page (order+1). */
        assert!(buddy_page.order == page.order);
        buddy_page.order += 1;
        page.order += 1;
        if page.start_addr() > buddy_page.start_addr() {
            page_ptr = buddy_page.start_addr() as *mut page;
        } else {
            page_ptr = page.start_addr() as *mut page;
        }

        return self.merge_page(&mut *page_ptr);
    }

    pub unsafe fn split_page(
        &mut self,
        page: &'static mut page,
        order: usize,
    ) -> &'static mut page {
        let page_ptr = page.start_addr() as *mut page;
        if page.order == order as u32 {
            return page;
        }

        page.order -= 1;
        let curr_order = page.order as usize;
        let buddy_page = self.find_buddy_chunk(&mut *page_ptr);
        buddy_page.order = page.order;
        buddy_page.allocated = false;
        buddy_page.next = self.list_heads[curr_order].take();
        self.list_heads[curr_order] = Some(buddy_page);

        return self.split_page(page, order);
    }

    pub unsafe fn free_pages(&mut self, page: &'static mut page) {
        assert!(page.allocated == true);
        page.allocated = false;

        let final_page = self.merge_page(page);
        final_page.allocated = false;

        // Insert into the corresponding free list
        let order = final_page.order as usize;
        final_page.next = self.list_heads[order].take();
        self.list_heads[order] = Some(final_page);
    }

    pub unsafe fn get_free_pages(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<&'static mut page> {
        let mut order = size_to_page_order(size);
        let needed_order = order as usize;

        while order < MAX_BUDDY_ORDER as u32 {
            match self.list_heads[order as usize].take() {
                Some(free_page) => {
                    self.list_heads[order as usize] = free_page.next.take();

                    if order as usize != needed_order {
                        let final_page = self.split_page(free_page, needed_order);
                        final_page.allocated = true;
                        return Some(final_page);
                    } else {
                        free_page.allocated = true;
                        return Some(free_page);
                    }
                }
                None => {
                    order += 1;
                }
            }
        }
        panic!("out of memory");
    }

    pub unsafe fn checkout_free_memory(&mut self) {
        serial_println!("--------------------");
        for order in 0..MAX_BUDDY_ORDER {
            let page = self.list_heads[order].take();
            let mut cnt = 0;
            if page.is_some() {
                cnt += 1;
                let first_ptr = page.unwrap().start_addr() as *mut page;
                let mut page_ptr = first_ptr;
                while let Some(ref mut p) = (*page_ptr).next {
                    cnt += 1;
                    page_ptr = p.start_addr() as *mut page;
                }
                self.list_heads[order] = Some(&mut *first_ptr);
            }
            serial_println!("order {} : {}", order, cnt);
        }
    }
}

unsafe impl GlobalAlloc for Locked<BuddyAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match allocator.get_free_pages(layout.size(), layout.align()) {
            Some(page) => allocator.page_to_virt(page) as *mut u8,
            None => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        let page = allocator.virt_to_page(ptr as usize);
        allocator.free_pages(page);
    }
}
