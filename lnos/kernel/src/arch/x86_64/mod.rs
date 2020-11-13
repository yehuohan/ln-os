//! x86_64 arch module
//!

use bootloader::BootInfo;

pub mod panic;


/// Kernel入口函数
///
/// _start函数使用C ABI调用，且不返回（`-> !`）；
#[no_mangle] // 禁止mangle函数名称
pub extern "C" fn _start(_boot_info: &'static BootInfo) -> ! {
    crate::kernel_main();
}
