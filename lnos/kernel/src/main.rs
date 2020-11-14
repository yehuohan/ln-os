//! lnos kernel程序
//!
//! 内核代码全部以库（liblnos）的形式提供；
//! 因为设置了no_main，所以执行cargo build时，会自动链接到liblnos中的入口函数_start；
//! 运行流程:
//! `_start() -> kernel_main() -> run(process)`
//!
//! 注意：cargo test时，liblnos是作为not(test)的库链接到main.rs，即main.rs不能使用liblnos中的测试框架。
//! （其实liblnos的测试，基本等同于lnos的测试，因为_start、panic等函数均是用liblnos中的）

#![no_std] // 禁用Rust标准库
#![no_main] // 禁用Rust标准程序入口（即禁用main入口，改成自己设定的，或默认的_start）


#[allow(unused_imports)]
use lnos;
