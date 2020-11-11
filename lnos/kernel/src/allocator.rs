//! 堆内存分配器

#![allow(unused_imports)]

pub mod bump;
pub mod linked_list;
pub mod fixed_size_block;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use linked_list_allocator::LockedHeap;
use bump::BumpAllocator;
use linked_list::LinkedListAllocator;
use fixed_size_block::FixedSizeBlockAllocator;
use x86_64::{
    structures::paging::{
        mapper::MapToError,
        FrameAllocator,
        Mapper,
        Page,
        PageTableFlags,
        Size4KiB,
    },
    VirtAddr,
};


#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());
//static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
//static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
//static ALLOCATOR: LockedHeap = LockedHeap::empty();
//static ALLOCATOR: Dummy = Dummy;

/// 设置堆地址
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// 设置堆大小
pub const HEAP_SIZE: usize = 100 * 1024;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}


pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc");
    }
}

/// 用于spin::Mutex的wrapper，用于规避孤儿规则（orphan rule）
pub struct Locked<T> {
    inner: spin::Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}

/// 按align对齐地址addr
///
/// align需要为2的幂次方
fn align_up(addr: usize, align: usize) -> usize {
    //let rest = addr % align;
    //if rest == 0 { addr } else { addr - rest + align }
    (addr + align - 1) & !(align - 1)
}
