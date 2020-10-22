
use x86_64::{
    structures::paging::{
        Page,
        PhysFrame,
        Mapper,
        Size4KiB,
        FrameAllocator,
        PageTable,
        OffsetPageTable},
    VirtAddr,
    PhysAddr,
};
use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType,
};


pub unsafe fn init(pmo: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(pmo);
    OffsetPageTable::new(level_4_table, pmo)
}

/// 获取L4页表虚拟地址
unsafe fn active_level_4_table(pmo: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read(); // CR3寄存器保存L4页表的物理地址
    let phys = level_4_table_frame.start_address();
    let virt = pmo + phys.as_u64(); // kernel已经在64位模式下运行，使用的是虚拟地址，如果要访问L4页表，需要将物理地址转成映射的虚拟地址
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}



/// -addr: 虚拟地址
/// -pmo: 物理偏移地址
pub unsafe fn translate_addr(addr: VirtAddr, pmo: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, pmo)
}

fn translate_addr_inner(addr: VirtAddr, pmo: VirtAddr) -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = pmo + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}

pub fn create_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
    ) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| {
            r.region_type == MemoryRegionType::Usable
        });
        let addr_ranges = usable_regions.map(|r| {
            r.range.start_addr() .. r.range.end_addr()
        });
        let frame_addresses = addr_ranges.flat_map(|r| {
            r.step_by(4096)
        });

        frame_addresses.map(|addr| {
            PhysFrame::containing_address(PhysAddr::new(addr))
        })
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
