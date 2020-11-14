use super::driver::vga;
use core::fmt::{Write, Arguments};

pub fn putfmt(args: Arguments) {
    use x86_64::instructions::interrupts;

    // 获取VGA的锁时，需要屏蔽中断，防止死锁
    #[cfg(not(test))]
    interrupts::without_interrupts(|| {
        vga::VGA
            .lock()
            .write_fmt(args)
            .unwrap();
    });

    #[cfg(test)]
    {
        use super::driver::serial;
        serial::SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Failed printing to serial(0x3F8)");
    }
}
