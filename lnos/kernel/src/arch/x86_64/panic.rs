//! Panic处理模块
//!
//! Rust在发生panic时，会调用panic_handler；
//! 对于kernel同样需要设置handler（并且对于build和test均需要设置）。

use core::panic::PanicInfo;


/// Kernel painc处理函数
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {}\n", info);

    #[cfg(test)]
    {
        use super::driver::acpi::{exit_qemu, QemuExitCode};
        exit_qemu(QemuExitCode::Failed);
    }

    loop {}
}
