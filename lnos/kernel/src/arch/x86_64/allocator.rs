//! 内存分配器模块
//!
//! 堆内存的分配，可以基于rust的堆内存管理来实现。
//! rust的alloc库的堆分配器需要实现GlobalAlloc。
//! 实现一个简单的Fixed Size Block Allocator。

use super::memory::GlobalFrameAllocator;
use super::memory::PageTableImpl;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr::{null_mut, NonNull}};
use x86_64::{
    structures::paging::{*, mapper::MapToError},
    VirtAddr,
};


/// 设置堆地址
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// 设置堆大小
pub const HEAP_SIZE: usize = 100 * 1024;

/// 设置alloc库的堆分配器
#[global_allocator]
static HEAP_ALLOCATOR: GlobalHeapLocker<GlobalHeapAllocator> = GlobalHeapLocker::new(GlobalHeapAllocator::new());

/// 设置alloc失败时的处理函数
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("global allocator error: {:?}", layout);
}

/// 用于spin::Mutex的wrapper，用于规避孤儿规则（orphan rule，无法为spin::Mutex实现GlobalAlloc）
pub struct GlobalHeapLocker<T> {
    inner: spin::Mutex<T>,
}

impl<T> GlobalHeapLocker<T> {
    pub const fn new(inner: T) -> Self {
        GlobalHeapLocker {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}

/// 链表内存块的大小
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct ListNode {
    next: Option<&'static mut ListNode>,
}

/// 从BLOCK_SIZES查找合适的内存块
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

/// kernel堆内存分配器
///
/// list_heads是内存块的链表，内存块支持BLOCK_SIZES中的大小；
/// 当list_heads不满足内存分配条件时，使用fallback来分配内存。
pub struct GlobalHeapAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback: linked_list_allocator::Heap,
}

impl GlobalHeapAllocator {
    pub const fn new() -> Self {
        Self {
            list_heads: [None; BLOCK_SIZES.len()],
            fallback: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => null_mut(),
        }
    }
}

/// 为GlobalHeapAllocator实现GlobalAlloc，作为rust的堆内存分配器
unsafe impl GlobalAlloc for GlobalHeapLocker<GlobalHeapAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        match list_index(&layout) {
            Some(index) => {
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        // 将链表的第1个节点取出作为alloc的内存
                        allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    },
                    None => {
                        // 最开始list_heads中全是None，未保存内存块，故需要fallback来分配内存，
                        // 且按照BLOCK_SIZES[index]分配的相应的内存块。
                        allocator.fallback_alloc(
                            Layout::from_size_align(BLOCK_SIZES[index], BLOCK_SIZES[index]).unwrap())
                    },
                }
            },
            None => allocator.fallback_alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        match list_index(&layout) {
            Some(index) => {
                // 回收内存时，将适合的内存块放回list_heads中，方便下次更块分配。
                let node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                // 回收的内存块处保存了ListNode数据，用于list_heads和ListNode.next索引
                let node_ptr = ptr as *mut ListNode;
                node_ptr.write(node);
                allocator.list_heads[index] = Some(&mut *node_ptr);
            },
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback.deallocate(ptr, layout);
            }
        }
    }
}


pub fn init() -> Result<(), MapToError<Size4KiB>> {
    let mut pg = PageTableImpl::active();
    let mut fa = GlobalFrameAllocator;

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::<Size4KiB>::containing_address(heap_start);
        let heap_end_page = Page::<Size4KiB>::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = fa
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            pg.mapper.map_to(page, frame, flags, &mut fa)?.flush()
        }
    }

    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}



#[test_case]
fn test_allocator() {
    use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

    let heap_value = Box::new(20);
    println!("heap value at {:p}", heap_value);
    assert_eq!(*heap_value, 20);

    let mut vec = Vec::new();
    for k in 0..128 {
        vec.push(k);
    }
    println!("vec at {:p}", vec.as_slice());
    assert_eq!(vec[0], 0);
    assert_eq!(vec[127], 127);

    let ref_cnt = Rc::new(vec![1, 2, 3]);
    let cloned_ref = ref_cnt.clone();
    println!("current ref cnt: {}", Rc::strong_count(&cloned_ref));
    assert_eq!(Rc::strong_count(&cloned_ref), 2);
    core::mem::drop(ref_cnt);
    println!("current ref cnt: {}", Rc::strong_count(&cloned_ref));
    assert_eq!(Rc::strong_count(&cloned_ref), 1)
}
