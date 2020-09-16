//! rcore kernel主程序入口
//!
//! OS Boot使用第三方库实现，这里的入口指程序已经进入内核空间运行。
//!

#![no_std] // 禁用Rust标准库
#![no_main] // 禁用Rust标准程序入口
#![feature(custom_test_frameworks)] // 自定义测试框架（使能#![test_runner]和#[test_case]）
#![test_runner(rcore::test_runner)] // 定义测试集运行函数
#![reexport_test_harness_main = "test_main"] // 定义测试框架入口函数


use core::panic::PanicInfo;

#[cfg(not(test))]
use rcore::println; // bin使用lib，需要使用lib的名称rcore；lib相互之间使用，可以用crate


/// Kernel入口函数
///
/// _start函数使用C ABI调用，且不返回（故有`-> !` 和 `loop{}`）；
/// crate bin需要自己的_start入口函数，包括crago run和cargo test模式下的。
#[cfg(not(test))]
#[no_mangle] // 禁止mangle函数名称
pub extern "C" fn _start() -> ! {
    rcore::init();

    // page fault
    //unsafe {
    //    *(0xdeadbeef as *mut u64) = 12;
    //};
    // double fault
    //fn statck_overflow() {
    //    statck_overflow();
    //}
    //statck_overflow();
    //loop {
    //    use rcore::print;
    //    for _ in 0..10000 {}
    //    print!("-"); // deadlock
    //}

    rcore::hlt_loop();
}

/// kernel的painc处理函数
///
/// 发生panic时调用；
/// crate bin需要自己的panic函数，包括crago run和cargo test模式下的。
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

/// cargo bin测试程序的入口函数
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    rcore::hlt_loop();
}

/// cargo bin测试程序的panic函数
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rcore::test_panic_handler(info);
}
