pub mod heap;

use limine::{memory_map::EntryType, response::MemoryMapResponse};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMapResponse,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMapResponse) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }
}

impl BootInfoFrameAllocator {
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let entries = self.memory_map.entries();
        let regions = entries.iter().filter(|r| r.entry_type == EntryType::USABLE);
        let addr_ranges = regions.map(|r| r.base..(r.base + r.length));
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

/// Map APIC base address to virtual memory
pub unsafe fn map_apic(
    mapper: &mut OffsetPageTable<'static>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), x86_64::structures::paging::mapper::MapToError<Size4KiB>> {
    let apic_phys = PhysAddr::new(0xFEE0_0000);
    let apic_page = Page::containing_address(VirtAddr::new(0xFEE0_0000));

    // Map APIC page as writeable
    unsafe {
        let _ = mapper.map_to(
            apic_page,
            PhysFrame::containing_address(apic_phys),
            x86_64::structures::paging::PageTableFlags::PRESENT
                | x86_64::structures::paging::PageTableFlags::WRITABLE
                | x86_64::structures::paging::PageTableFlags::NO_EXECUTE,
            frame_allocator,
        )?;
    }

    Ok(())
}
