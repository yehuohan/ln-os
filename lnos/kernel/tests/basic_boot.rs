//! 启动测试
//!
//! cargo test将tests下每个rs文件当成一个单独的crate编译，故每个rs文件需要自己的_start和panic函数

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lnos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use lnos::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lnos::test_panic_handler(info);
}

#[test_case]
fn test_println() {
    println!("This is println");
}
