//! arch模块
//!
//! 基于x86_64实现arch模块。
//!

use bootloader::BootInfo;

pub mod panic;
pub mod driver;
pub mod io;


/// Kernel入口函数
///
/// _start是rust程序编译时默认的入口；
/// 为了将_start作为kernelt入口函数，需要使用C ABI调用，且不返回（`-> !`）；
#[cfg(not(test))]
#[no_mangle] // 禁止mangle函数名称
pub extern "C" fn _start(_boot_info: &'static BootInfo) -> ! {
    println!("Hello lnos!");

    driver::init();

    crate::kernel_main();

    hlt_loop();
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start(_boot_info: &'static BootInfo) -> ! {
    println!("Running liblnos test");
    crate::test_main();

    use driver::acpi::{exit_qemu, QemuExitCode};
    exit_qemu(QemuExitCode::Success);

    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
