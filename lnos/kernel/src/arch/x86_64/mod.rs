//! arch模块
//!
//! 基于x86_64实现arch模块。
//!

use bootloader::BootInfo;

pub mod panic;
pub mod driver;
pub mod io;
pub mod gdt;
pub mod idt;
pub mod pic;
pub mod memory;
pub mod allocator;


/// Kernel入口函数
///
/// _start是rust程序编译时默认的入口；
/// 为了将_start作为kernelt入口函数，需要使用C ABI调用，且不返回（`-> !`）；
//#[no_mangle] // 禁止mangle函数名称
//pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
pub fn kernel_start(boot_info: &'static BootInfo) -> ! {
    println!("Hello lnos!");

    memory::init(&boot_info);
    allocator::init().expect("failed to init allocator");
    gdt::init();
    idt::init();
    pic::init();

    x86_64::instructions::interrupts::enable(); // 使能中断

    crate::kernel_main();

    hlt_loop();
}

/// kernel测试程序入口
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    println!("Running liblnos test");

    memory::init(&boot_info);
    allocator::init().expect("failed to init allocator");
    gdt::init();
    idt::init();
    pic::init();

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
