use core::fmt;

use super::BUFFER;
use font8x8::legacy::BASIC_LEGACY;
use lazy_static::lazy_static;
use spin::Mutex;

pub struct Writer {
    color: u32,
    row: u64,
    col: u64,
    height: u64,
    width: u64,
}

impl Writer {
    pub fn new(color: u32) -> Self {
        let height = 1080 / 2 - 4;
        let width = (1920 - 1600) / 2;
        Self {
            color,
            row: 0,
            col: 0,
            height,
            width,
        }
    }

    pub fn get_width(&self) -> u64 {
        self.width * 2
    }

    pub fn change_color(&mut self, color: u32) {
        self.color = color;
    }

    pub fn write_pixel(&self) {
        let x = self.row;
        let y = self.col;

        let rs = 2 * x;
        let cs = 2 * y;
        let re = rs + 2;
        let ce = cs + 2;

        let buffer = &BUFFER;

        for x in rs..re {
            for y in cs..ce {
                buffer.write_pixel(x, y, self.color);
            }
        }
    }

    pub fn write_char(&mut self, ch: char) {
        if ch == '\n' {
            self.new_line();
            return;
        }

        let bitmap: [u8; 8] = BASIC_LEGACY[ch as usize];
        let ox = self.row;
        let oy = self.col;

        for row in 0..8 {
            for col in 0..8 {
                if (bitmap[row] >> col) & 1 != 0 {
                    self.write_pixel();
                }
                self.col += 1;
            }
            self.row += 1;
            self.col = oy;
        }

        self.row = ox;
        self.col = oy + 8;

        if self.col >= self.width {
            self.new_line();
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    fn new_line(&mut self) {
        self.col = 0;
        self.row += 8;

        if self.row >= self.height {
            self.scroll();
            self.row -= 8;
        }
    }

    fn scroll(&self) {
        let buffer = &BUFFER;
        let scale = buffer.fb.height() / self.height as u64;
        buffer.scroll_lines(scale * 8);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new(0xFFFFFFFF));
}
