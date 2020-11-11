//! lnos kernel主程序入口
//!
//! OS Boot使用第三方库实现，这里的入口指程序已经进入kernel空间运行。
//!

#![no_std] // 禁用Rust标准库
#![no_main] // 禁用Rust标准程序入口
#![feature(custom_test_frameworks)] // 自定义测试框架（使能#![test_runner]和#[test_case]）
#![test_runner(lnos::test_runner)] // 定义测试集运行函数
#![reexport_test_harness_main = "test_main"] // 定义测试框架入口函数


extern crate alloc; // 编译链接alloc库（属于rust标准库）
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};


entry_point!(kernel_main); // 设置kernel入口函数

/// Kernel入口函数
///
/// _start函数使用C ABI调用，且不返回（故有`-> !` 和 `loop{}`）；
/// crate bin需要自己的_start入口函数，包括crago run和cargo test模式下的。
//#[cfg(not(test))]
//#[no_mangle] // 禁止mangle函数名称
//pub extern "C" fn _start() -> ! {
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use lnos::memory;
    use lnos::allocator;
    use x86_64::{structures::paging::{MapperAllSizes, Page}, VirtAddr};

    lnos::println!("Hello lnos!");
    lnos::init();

    // page fault
    unsafe {
        let _x = *(0x20478b as *mut u32);
        //*(0x20478b as *mut u32) = 12;
        //*(0xdeadbeef as *mut u32) = 12;
    };
    // double fault
    //fn statck_overflow() {
    //    statck_overflow();
    //}
    //statck_overflow();
    //loop {
    //    use lnos::print;
    //    for _ in 0..10000 {}
    //    print!("-"); // deadlock
    //}

    /* memory */
    let pmo = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(pmo) };
    //let mut frame_allocator = memory::EmptyFrameAllocator;
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    //let page = Page::containing_address(VirtAddr::new(0));
    let page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
    memory::create_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe {
        page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e); // New!
    }

    let addresses = [
        0xb8000,
        0x201008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset,
    ];
    for &addr in &addresses {
        let virt = VirtAddr::new(addr);
        let phys = mapper.translate_addr(virt);
        lnos::println!("{:?} -> {:?}", virt, phys);
    }

    let heap_value = Box::new(20);
    lnos::println!("heap value at {:p}", heap_value);
    let mut vec = Vec::new();
    for k in 0..500 {
        vec.push(k);
    }
    lnos::println!("vec at {:p}", vec.as_slice());

    let ref_cnt = Rc::new(vec![1, 2, 3]);
    let cloned_ref = ref_cnt.clone();
    lnos::println!("current ref cnt: {}", Rc::strong_count(&cloned_ref));
    core::mem::drop(ref_cnt);
    lnos::println!("current ref cnt: {}", Rc::strong_count(&cloned_ref));

    lnos::hlt_loop();
}

/// kernel的painc处理函数
///
/// 发生panic时调用；
/// crate bin需要自己的panic函数，包括crago run和cargo test模式下的。
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lnos::println!("{}", info);
    loop {}
}

/// cargo bin测试程序的入口函数
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    lnos::hlt_loop();
}

/// cargo bin测试程序的panic函数
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lnos::test_panic_handler(info);
}
