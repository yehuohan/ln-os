//! Kernel库模块
//!
//! 将lnos的整个kernel作为一个库实现。
//!

#![no_std] // 禁用Rust标准库


/// 设置arch
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
pub mod arch;


pub fn kernel_main() -> ! {
    loop {
    }
}
