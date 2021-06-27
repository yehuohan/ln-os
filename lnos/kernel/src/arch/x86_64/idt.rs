//! IDT模块
//!
//! 设置CPU Exception的中断服务例程；
//! 实际操作为，使用`lidt`指令，将数据IDT的地址和长度保存在IDTR寄存器。
//! IDT在数据上来说，本质是一个uint8[256][16]数组，每16bytes是一个Entry。

use super::gdt;
use super::{pic, pic::PicIRQ};
use lazy_static::lazy_static;
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode,
};


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // 设置exception中断例程
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[PicIRQ::Timer.as_usize()].set_handler_fn(pic::timer_handler);
        idt[PicIRQ::Keyboard.as_usize()].set_handler_fn(pic::keyboard_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}


/// Breakpoint(No = 3)可以暂停程序执行，常用于调试；
/// 使用指令int 3（3为Breakpoint的中断号）可以触发Breakpoint。
/// x86_64 crate中可以使用instructions::interrupts::int3()测试。
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: Breakpoint\n{:#?}", stack_frame);
}

/// Page Fault(No = 14)，当访问的地址没有对应的物理地址，会触发Page Fault。
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: Page Fault");
    println!("Accessed Address: {:?}", Cr2::read()); // CR2寄存器保存了引起中断的Virtual Address
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);

    // 没有为Virtual Address映射Physical Address前，代码应当停止运行
    super::hlt_loop();
}


/// Double Fault(No = 8)异常处理
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("EXCEPTION: Double Fault({})\n{:#?}", error_code, stack_frame);
}



#[test_case]
fn test_breakpoint() {
    x86_64::instructions::interrupts::int3();
}

// 触发Double Fault后，test_double_fault没法正常返回，所以测试会失败，
// 可以在_start中调用test_double_fault观察结果。
#[allow(dead_code)]
//#[test_case]
pub fn test_double_fault() {
    // 调用stack_overflow导致kernel栈溢出，触发Page Fault，
    // 因为栈溢出了，不能正确调用Page Fault的Handler（因为Guard Page），
    // 然后触发Double Fault，处理Double Fault时，会先切换到IST[0]中的栈，
    // 保证可以正确调用Double Fault的Handler。
    #[allow(unconditional_recursion)]
    fn stack_overflow() {
        stack_overflow();
        volatile::Volatile::new(0).read(); // 防止尾递归优化
    }
    stack_overflow();
}

// 触发Double Fault后，test_page_fault没法正常返回，所以测试会失败，
#[allow(dead_code)]
//#[test_case]
pub fn test_page_fault() {
    // 修改deadbeef地址的内存，会触发Page Fault，
    // 若没有设置Page Fault的Handler，则会触发Double Fault，
    // 因为此时栈是正常的，所以可以正确调用Double Fault的Handler。
    // 反之，则触发Page Fault的Handler。
    unsafe {
        *(0xdeadbeef as *mut u64) = 10;
    }
}
