//! PIC模块
//!
//! 使用经共8259作为PIC；
//!
//! TODO 使用APIC替代8259

use spin;
use pic8259::ChainedPics;
use x86_64::structures::idt::InterruptStackFrame;


/// Primary PIC起始中断号
pub const PIC_1_OFFSET: u8 = 32;

/// Secondary PIC起始中断号
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Primary和Secondary中断控制器；
/// PICS总共可以管理16个中断，中断号为32~47。
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(
    unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// PIC中断号（Interrupt Request Number）
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PicIRQ {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl PicIRQ {
    pub fn as_u8(self) -> u8 { self as u8 }
    pub fn as_usize(self) -> usize { usize::from(self.as_u8()) }
}


pub fn init() {
    unsafe { PICS.lock().initialize(); };
}


/// Timer中断(No = 32)
pub extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        // 通知PIC，已经完成中断处理，不然无法响应下一个中断
        PICS.lock().notify_end_of_interrupt(PicIRQ::Timer.as_u8());
    }
}

/// Keyboard中断(No = 33)
pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    /* 直接在中断中读取按键码，并处理按键
    use spin::Mutex;
    use lazy_static::lazy_static;
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                    layouts::Us104Key,
                    ScancodeSet1,
                    HandleControl::Ignore));
    }
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60); // 通过端口0x60读取PS/2 controller的数据
    let scancode: u8 = unsafe { port.read() }; // 读取按键scancode

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(PicIRQ::Keyboard.as_u8());
    }
    */

    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60); // 通过端口0x60读取PS/2 controller的数据
    let scancode: u8 = unsafe { port.read() }; // 读取按键scancode
    crate::driver::keyboard::append_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(PicIRQ::Keyboard.as_u8());
    }
}
