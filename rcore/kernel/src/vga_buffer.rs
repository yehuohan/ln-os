//! 标准VGA(0xB8000)显示驱动

use core::fmt;
use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;


/// 颜色变量
#[allow(dead_code)] // 禁止提示未使用变量
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // enum值使用u8保存
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// VGA颜色值
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // 保证ColorCode和u8的data layout, ABI是一样的
struct ColorCode(u8);

impl ColorCode {
    /// 生成VGA颜色码
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

/// 屏幕显示字符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // 使用C语言的data layout策略
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// VGA屏幕高度
const BUFFER_HEIGHT: usize = 25;
/// VGA屏幕宽度
const BUFFER_WIDTH: usize = 80;

/// VGA缓存
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// VGA输出对象
pub struct Writer {
    column_position: usize, // 当前光标位置
    color_code: ColorCode, // 当前颜色
    buffer: &'static mut Buffer, // 'static指定buffer的生命周期和整个程序相同
}

impl Writer {
    /// 输出一个字符
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// 换行
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let charater = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(charater);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// 清除一行
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// 输出字符串
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    /// 实现标准格化输出
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}


lazy_static! {
    /// VGA全局输出对象
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Red, Color::Black),
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // 获取WRITER的锁时，屏蔽中断
    interrupts::without_interrupts(|| {
        WRITER.lock()
              .write_fmt(args)
              .unwrap();

    });
}

/// 实现print宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// 实现println宏
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}


/// 测试println
#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer
                                    .chars[BUFFER_HEIGHT - 2][i]
                                    .read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
