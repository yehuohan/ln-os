use super::driver::vga;
use core::fmt::{Write, Arguments};

pub fn putfmt(args: Arguments) {
    use x86_64::instructions::interrupts;

    // 获取VGA的锁时，需要屏蔽中断，防止死锁
    interrupts::without_interrupts(|| {
        vga::VGA
            .lock()
            .write_fmt(args)
            .unwrap();
    });
}
