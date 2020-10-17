//! kernel模块
//!
//! 将lnos的整个kernel作为一个库实现。
//!

#![no_std]
#![cfg_attr(test, no_main)] // 生成测试程序时，禁用Rust标准程序入口
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]


use core::panic::PanicInfo;

// 编译链接第三方crate lib
extern crate rlibc;

// 声明模块，内容在src/<mod>.rs或src/<mod>/mod.rs文件中
pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// kernel初始化函数
pub fn init() {
    println!("Hello lnos!");
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize(); };
    x86_64::instructions::interrupts::enable(); // 使能中断
}


/// 执行测试用例的trait
pub trait Testable {
    fn run(&self) -> ();
}

/// 为Fn()实现Testable
impl<T> Testable for T
    where T: Fn() {
    fn run(&self) {
        // 自定义每条测试用例执行前后的打印内容
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[Ok]");
    }
}

/// 测试集运行函数
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// 测试程序的panic处理函数
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[Failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// qemu退出码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// 退出qemu程序
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}


/// cargo test的入口函数
///
/// cargo test模式下，crate lib会编译出一个测试bin文件，故需要自己的_start入口函数
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop();
}

/// cargo test的panic处理函数
///
/// cargo test模式下，crate lib会编译出一个测试bin文件，故需要自己的panic函数
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}
