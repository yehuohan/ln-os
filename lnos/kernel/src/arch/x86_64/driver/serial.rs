use spin::Mutex;
use lazy_static::lazy_static;
use uart_16550::SerialPort;

lazy_static! {
    /// 第一个串口设备（默认标准地址为0x3F8）
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}
