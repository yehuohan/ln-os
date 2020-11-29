//! 内存管理模块
//!
//! bootloader在引导kernel前，已经做好分页表了；
//! 并且将内存相关数据放到BootInfo，传递给kernel了。
//!
//! bootloader使用Map the Complete Physical Memory对页表映射；
//! 物理起始地址为0，对应的虚拟地起地址为physical_memory_offset；
//! 地址本质是一个usize（CPU的地址宽度等于usize的位宽），可以作
//! 为VirtAddr或PhysAddr使用（类似于c中，将usize赋值给指针变量）。
//! memory_map保存了内存的起始地址、类型的等信息。
//!

use spin::Mutex;
use bitmap_allocator::BitAlloc;
use bootloader::{BootInfo, bootinfo::MemoryRegionType};
use x86_64::{
    instructions::interrupts,
    structures::paging::*,
    VirtAddr, PhysAddr,
};


type FrameBitAlloc = bitmap_allocator::BitAlloc64K; // 64*1024*4K=256M

/// 通过bitmap可用的物理内存Frame进行管理
static FRAME_ALLOCATOR: Mutex<FrameBitAlloc> = Mutex::new(FrameBitAlloc::DEFAULT);

/// 内存映射偏移地址
static mut PHYS_MEM_OFS: u64 = 0x0;

/// 内存管理初始化
///
/// 在调用memory::init后，才能使用PageTableImpl、GlobalFrameAllocator等；
pub fn init(boot_info: &'static BootInfo) {
    let phys_mem_ofs = VirtAddr::new(boot_info.physical_memory_offset);
    let phys_mem_map = &boot_info.memory_map;
    println!("Physical Memory Offset: {:?}", phys_mem_ofs);
    println!("Physical Memory Region:");
    for i in phys_mem_map.iter() {
        println!("    0x{:08x} -> 0x{:08x} : {:?}",
            i.range.start_addr(), i.range.end_addr(), i.region_type);
    }

    unsafe {
        PHYS_MEM_OFS = boot_info.physical_memory_offset;
    }

    // 初始化Frame分配器，标记可用内存地址；
    // 此时还没有使能中断，所以使用FRAME_ALLOCATOR时，不用关中断；
    let mut fa = FRAME_ALLOCATOR.lock();
    for i in phys_mem_map
                .iter()
                .filter(|r| r.region_type == MemoryRegionType::Usable)
                .into_iter() {
        // bootloader已经将Usable的内存按4K对齐了，可以直接标记
        fa.insert((i.range.start_frame_number as usize) .. (i.range.end_frame_number as usize));
    }
}

/// 页表映射实现
pub struct PageTableImpl {
    pub mapper: OffsetPageTable<'static>,
}

impl PageTableImpl {
    /// 获取页表映射实例
    pub fn active() -> Self {
        use x86_64::registers::control::Cr3;
        let l4_frame: PhysFrame = Cr3::read().0; // CR3保存L4的物理Frame
        let phys = l4_frame.start_address(); // l4_frame的物理地址
        unsafe {
            let phys_mem_ofs = VirtAddr::new(PHYS_MEM_OFS);
            let virt = phys_mem_ofs + phys.as_u64(); // 访问L4的虚拟地址
            let l4_page: *mut PageTable = virt.as_mut_ptr(); // L4的虚拟Page
            PageTableImpl{
                mapper: OffsetPageTable::new(&mut *l4_page, phys_mem_ofs),
            }
        }
    }

    /// 实现page到frame的映射
    pub fn map(&mut self, page: Page::<Size4KiB>, frame: PhysFrame::<Size4KiB>) {
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let mut frame_allocator = GlobalFrameAllocator;
        let map_to_result = unsafe {
            self.mapper.map_to(page, frame, flags, &mut frame_allocator)
        };
        map_to_result.expect("PageTableImpl::map failed").flush();
    }
}

/// Frame分配器
///
/// 这是一个简单的Frame分配器，只支持4KiB大小的Frame。
#[derive(Debug, Clone, Copy)]
pub struct GlobalFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
    /// 申请一个4KiB的物理Frame
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        interrupts::without_interrupts(|| {
            if let Some(n) = FRAME_ALLOCATOR.lock().alloc() {
                // FRAME_ALLOCATOR返回的地址再乘上Size4KiB一定是4KiB对齐的，故无需check
                unsafe {
                    Some(PhysFrame::from_start_address_unchecked(PhysAddr::new(n as u64 * Size4KiB::SIZE)))
                }
            } else {
                None
            }
        })
    }
}

impl FrameDeallocator<Size4KiB> for GlobalFrameAllocator {
    /// 释放一个4KiB的物理Frame
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        interrupts::without_interrupts(|| {
            FRAME_ALLOCATOR
                .lock()
                .dealloc(frame.start_address().as_u64() as usize / Size4KiB::SIZE as usize);
        })
    }
}



/// 打印内存相关信息
#[test_case]
fn test_memory_info() {
    unsafe {
        let pmo = VirtAddr::new(PHYS_MEM_OFS);
        print_page_table(pmo);
        print_virt2phys_translation(pmo);
        print_create_mapping();
        print_check_frame();
    }
}

/// 打印页表Entry对应Frame的物理地址
#[allow(dead_code)]
unsafe fn print_page_table(pmo: VirtAddr) {
    use x86_64::registers::control::Cr3;
    let l4_frame: PhysFrame = Cr3::read().0; // CR3保存L4的物理Frame

    // 为了访问页表中的Entry，需要知道访问页表的虚拟地址
    println!("Physical Address of L4: {:?}", l4_frame.start_address());
    let phys = l4_frame.start_address(); // l4_frame的物理地址
    let virt = pmo + phys.as_u64(); // 访问L4的虚拟地址
    let l4_page: *mut PageTable = virt.as_mut_ptr(); // L4的虚拟Page

    for (i, entry) in (*l4_page).iter().enumerate() { // 遍历L4的Entry
        if !entry.is_unused() {
            println!("L4 Entry[{}]: {:?}:", i, entry.addr()); // Entry中保存的物理地址
            let l3_frame: PhysFrame = entry.frame().unwrap(); // L3的物理Frame（即entry.addr()对应的Frame）
            let phys = l3_frame.start_address(); // l3_frame的物理地址
            let virt = pmo + phys.as_u64();
            let l3_page: *mut PageTable = virt.as_mut_ptr();

            for (i, entry) in (*l3_page).iter().enumerate() { // 遍历L3的Entry
                if !entry.is_unused() {
                    println!("    L3 Entry[{}]: {:?}", i, entry.addr());
                }
            }
        }
    }
}

/// 虚拟地址到物理地址的计算
#[allow(dead_code)]
fn print_virt2phys_translation(pmo: VirtAddr) {
    let addresses = [
        0xb8000, // VGA地址
        0x201008, // code页
        0x0100_0020_1a10, // stack页
    ];

    for &addr in &addresses {
        let virt = VirtAddr::new(addr);

        use x86_64::registers::control::Cr3;
        let mut frame: PhysFrame = Cr3::read().0; // L4

        for &idx in &[virt.p4_index(), virt.p3_index(), virt.p2_index(), virt.p1_index()] {
            let frame_virt = pmo + frame.start_address().as_u64();
            let table = unsafe { &*(frame_virt.as_ptr() as *const PageTable) };
            let entry = &table[idx];
            frame = entry.frame().unwrap(); // L4 -> L3 -> L2 -> L1
            //use x86_64::structures::paging::page_table::FrameError;
            //frame = match entry.frame() {
            //    Ok(frame) => frame,
            //    Err(FrameError::FrameNotPresent) => None,
            //    Err(FrameError::HugeFrame) => panic!("huge pages not supported"), // frame()方法不支持huge page
            //};
        }
        let phys = frame.start_address() + u64::from(virt.page_offset());

        println!("{:?} -> {:?}", virt, phys);
    }

    // pmo对应物理起始地址（即0x00）
    let pg = PageTableImpl::active();
    let virt = VirtAddr::new(pmo.as_u64());
    let phys = pg.mapper.translate_addr(virt);
    println!("{:?} -> {:?}", virt, phys);
    assert_eq!(0 as u64, phys.unwrap().as_u64());
}

/// 创建一个Page到Frame的映射，大小为4KiB
#[allow(dead_code)]
fn print_create_mapping() {
    use super::driver::vga::{BUF_ADDR, BUF_ROW, BUF_COL};
    // 将page映射到frame
    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0)); // 4KiB的page，包含地址0（会将地址对行4K对齐）
    let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(BUF_ADDR)); // 4Kib的frame，包含vga地址（会将地址对行4K对齐）
    let mut pg = PageTableImpl::active();
    pg.map(page, frame);

    let vga_ptr: *mut u16 = page.start_address().as_mut_ptr();
    unsafe {
        (vga_ptr as *mut u64).offset((BUF_COL * 4) as isize).write_volatile(0x_b021_f077_f065_f04e); // New!
        vga_ptr.offset((BUF_COL * BUF_ROW / 2 - 2) as isize).write_volatile(0x_f072); // r
        vga_ptr.offset((BUF_COL * BUF_ROW / 2 - 1) as isize).write_volatile(0x_f071); // q
        vga_ptr.offset((BUF_COL * BUF_ROW / 2 + 0) as isize).write_volatile(0x_b021); // !
        vga_ptr.offset((BUF_COL * BUF_ROW / 2 + 1) as isize).write_volatile(0x_f071); // q
        vga_ptr.offset((BUF_COL * BUF_ROW / 2 + 2) as isize).write_volatile(0x_f072); // r
    }

    // 校验新映射的VGA地址的内容
    #[cfg(test)]
    x86_64::instructions::interrupts::without_interrupts(|| {
        use super::driver::vga::VGA;
        let vga = VGA.lock();
        assert_eq!('r', vga.read_byte(BUF_ROW / 2, BUF_COL / 2 - 2));
        assert_eq!('q', vga.read_byte(BUF_ROW / 2, BUF_COL / 2 - 1));
        assert_eq!('!', vga.read_byte(BUF_ROW / 2, BUF_COL / 2 + 0));
        assert_eq!('q', vga.read_byte(BUF_ROW / 2, BUF_COL / 2 + 1));
        assert_eq!('r', vga.read_byte(BUF_ROW / 2, BUF_COL / 2 + 2));
    });
}

/// 测试物理Frame有效性
#[allow(dead_code)]
fn print_check_frame() {
    if let Some(fa) = FRAME_ALLOCATOR.try_lock() {
        for &addr in &[0x430, 0x780, 0x7fdf, 0x7fe0] {
            println!("0x{:x} : {}", addr, fa.test(addr));
        }
    }
}
