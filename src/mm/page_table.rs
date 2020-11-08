use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::{PhysAddr, VirtAddr};
/// A 64-bit page table entry.
// #[derive(Clone)]
// #[repr(transparent)]
// pub struct PageTableEntry {
//     entry: u64,
// }

// #[repr(align(4096))]
// #[repr(C)]
// pub struct PageTable {
//     entries: [PageTableEntry; ENTRY_COUNT],
// }

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_page_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_page_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level4_frame, _) = Cr3::read();
    let paddr = level4_frame.start_address();

    let vaddr = physical_memory_offset + paddr.as_u64();

    let page_table_ptr: *mut PageTable = vaddr.as_mut_ptr();

    &mut *page_table_ptr
}

/// Return a virtual address of a given physical address used by kernel
#[allow(dead_code)]
pub fn phys_to_virt(paddr: PhysAddr, physical_memory_offset: VirtAddr) -> VirtAddr {
    VirtAddr::new(physical_memory_offset.as_u64() + paddr.as_u64())
}

/// Translates the given virtual address to the mapped physical address, or
/// `None` if the address is not mapped.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`.
#[allow(dead_code)]
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    // read the active level 4 frame from the CR3 register
    let (level4_table, _) = Cr3::read();
    let mut table_frame = level4_table;

    let table_index = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];

    // traverse the multi-level page table
    for &index in &table_index {
        let vaddr = physical_memory_offset + table_frame.start_address().as_u64();
        let page_table_ptr: *const PageTable = vaddr.as_ptr();
        let page_table: &PageTable = unsafe { &*page_table_ptr };

        let pte = &page_table[index];
        table_frame = match pte.frame() {
            Ok(table_frame) => table_frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge page not supported!"),
        };
    }

    Some(PhysAddr::new(
        table_frame.start_address().as_u64() + u64::from(addr.page_offset()),
    ))
}
