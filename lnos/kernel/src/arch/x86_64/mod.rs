//! # arch模块
//!
//! 基于x86_64实现arch模块。
//!

use bootloader::BootInfo;

pub mod panic;
pub mod driver;
pub mod io;

/// Kernel入口函数
///
/// _start函数使用C ABI调用，且不返回（`-> !`）；
#[no_mangle] // 禁止mangle函数名称
pub extern "C" fn _start(_boot_info: &'static BootInfo) -> ! {
    println!("Hello lnos!");

    driver::init();

    crate::kernel_main();
}
