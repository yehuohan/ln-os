//! 将退出qemu当成简单的电源关机处理
//!
//! 对于qemu：
//! - 通过操作isa-debug-exit设备来实现qemu的退出；
//! - 通过serial设备，访问qemu的屏幕（stdio）内容，并输出到Host主机终端；


/// qemu退出码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// 退出qemu
pub fn exit_qemu(ecode: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(ecode as u32);
    }
}
