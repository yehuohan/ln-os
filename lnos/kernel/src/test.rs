//! lnos kernel test模块
//!
//! 基于lnos kernel实现测试框架。
//!
//! 执行cargo test时，会生成2个测试程序：
//! - 将lib.rs库代码中所有test_case函数，链接到测试程序执行；
//! - 将main.rs程序代码中所有test_case函数，链接到测试程序行；
//!
//! 测试程序中的入口函数和panic_handler是独立的；
//! liblnos设置了no_main属性，所以需要为测试程序提供_start和panic_handler，
//! 然后依次调用：
//! `_start() -> test_main() -> test_runner()`
//!


/// 执行测试用例的trait
pub trait Testable {
    fn run(&self) -> ();
}

/// 为Fn()实现Testable
impl<T> Testable for T
where T: Fn()
{
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("[Ok]");
    }
}

/// 测试集运行函数
///
/// runner将依次执行数组中的每个闭包i（dyn Fn()）
pub fn runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
}



#[test_case]
fn it_works() {
    assert!(true);
    assert_eq!(1 + 2, 3);
}
