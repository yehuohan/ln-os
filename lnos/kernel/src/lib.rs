//! lnos kernel库模块
//!
//! 将lnos的整个kernel作为一个库实现。
//!

#![no_std] // 禁用Rust标准库
#![cfg_attr(test, no_main)] // 生成测试程序时，禁用Rust标准程序入口
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![feature(custom_test_frameworks)] // 自定义测试框架（使能#![test_runner]和#[test_case]）
#![test_runner(crate::test::runner)] // 定义测试集运行函数
#![reexport_test_harness_main = "test_main"] // 定义测试框架入口函数


extern crate rlibc; // 需要使用rlibc中的memcpy、memset等函数

#[macro_use]
pub mod console;
pub mod test;

// 设置arch
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
pub mod arch;


pub fn kernel_main() {
}
