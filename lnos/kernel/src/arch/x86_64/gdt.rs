//! GDT模块
//!
//! GDT的初始化是使用lgdt指令，将GDT的地址和长度，加载GDTR寄存器。

use x86_64::VirtAddr;
use x86_64::structures::gdt::{
    GlobalDescriptorTable,
    Descriptor,
};
use x86_64::structures::tss::TaskStateSegment;



/// Double Fault的中断栈帧保存在IST[0]处
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Double Fault的中断栈帧大小
pub const DOUBLE_FAULT_IST_STACK_SIZE: usize = 0x1000 * 5;

/// 一个Cpu Core需要的相关数据
pub struct Cpu {
    gdt: GlobalDescriptorTable,
    tss: TaskStateSegment,
    double_fault_stack: [u8; DOUBLE_FAULT_IST_STACK_SIZE],
}

impl Cpu {
    unsafe fn init(&'static mut self) {
        use x86_64::instructions::segmentation::set_cs;
        use x86_64::instructions::tables::load_tss;

        // 设置Double Fault的IST
        let stack_top = VirtAddr::from_ptr(&self.double_fault_stack) + DOUBLE_FAULT_IST_STACK_SIZE;
        self.tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_top;

        // 设置Selector
        let code_selector = self.gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = self.gdt.add_entry(Descriptor::tss_segment(&self.tss));
        self.gdt.load();
        set_cs(code_selector);
        load_tss(tss_selector);
    }
}

static mut CPUS: Cpu = Cpu {
    gdt: GlobalDescriptorTable::new(),
    tss: TaskStateSegment::new(),
    double_fault_stack: [0u8; DOUBLE_FAULT_IST_STACK_SIZE],
};

pub fn init() {
    unsafe {
        CPUS.init();
    }
}
