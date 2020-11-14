//! console模块
//!
//! 实现控制台的基本输入输出


/// 基本的print宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::arch::io::putfmt(format_args!($($arg)*)));
}

/// 基本的println宏
#[cfg(not(test))]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
#[cfg(test)]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
            concat!($fmt, "\n"), $($arg)*));
}
