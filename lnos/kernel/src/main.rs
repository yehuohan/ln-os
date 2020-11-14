//! lnos kernel程序
//!
//! 内核代码全部以库（libkernel）的形式提供；
//! 将libkernel编译到二进制文件时，将会链接到指定的入口函数。
//!
//! 运行流程: _start() -> kernel_main() -> run(process)

#![no_std] // 禁用Rust标准库
#![no_main] // 禁用Rust标准程序入口


#[allow(unused_imports)]
use lnos;
