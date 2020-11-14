//! # VGA模块
//!
//! 实现基本打印输出

use core::fmt;
use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;


lazy_static! {
    /// VGA全局输出对象
    pub static ref VGA: Mutex<Vga> = Mutex::new(Vga {
        col: 0,
        attr: ColorCode::new(Color::Red, Color::Black),
        buf: unsafe { &mut *(0xB8000 as *mut Buffer) },
    });
}


/// 屏幕高度
const BUF_ROW: usize = 25;
/// 屏幕宽度
const BUF_COL: usize = 80;

/// 颜色值
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

/// 颜色码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // 保证ColorCode和u8的data layout, ABI是一样的
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

/// VGA字符显示
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // 使用C语言的data layout策略
struct CharCell {
    /// 单个ascii字符
    schar: u8,
    /// 字符颜色属性
    color: ColorCode,
}

/// VGA显示缓存
#[repr(transparent)]
struct Buffer {
    cells: [[Volatile<CharCell>; BUF_COL]; BUF_ROW],
}

/// VGA打印
pub struct Vga {
    // 光标当前行
    //row: usize,
    /// 光标当前列
    col: usize,
    /// 当前使用的属性（只有颜色属性）
    attr: ColorCode,
    /// 字符显示缓存
    buf: &'static mut Buffer,
}

impl Vga {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col >= BUF_COL { self.new_line(); }
                self.buf.cells[BUF_ROW - 1][self.col].write(CharCell {
                    schar: byte,
                    color: self.attr,
                });
                self.col += 1;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUF_ROW {
            for col in 0..BUF_COL {
                let charater = self.buf.cells[row][col].read();
                self.buf.cells[row - 1][col].write(charater);
            }
        }
        self.clear_row(BUF_ROW - 1);
        self.col = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = CharCell {
            schar: b' ',
            color: self.attr,
        };
        for col in 0..BUF_COL {
            self.buf.cells[row][col].write(blank);
        }
    }
}

impl fmt::Write for Vga {
    /// 实现标准格式化输出
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
