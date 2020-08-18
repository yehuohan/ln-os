
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rcore::{QemuExitCode, exit_qemu, serial_println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
        serial_println!("[Test did NOT panic]"); // 正常运行到这一行，则说明没有触发panic函数
        exit_qemu(QemuExitCode::Failed);
    }
    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[Ok]"); // 触发panic函数，说明发生了panic，即should_panic成功
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[test_case]
fn should_panic() {
    assert_eq!(1, 3); // 会触发panic函数
}
