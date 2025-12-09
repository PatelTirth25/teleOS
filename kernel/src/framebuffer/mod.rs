pub mod fps;
pub mod screen;
pub mod writer;

use core::fmt;

use crate::{boot::boot_info, serial_println};
use lazy_static::lazy_static;
use limine::framebuffer::Framebuffer;
use writer::WRITER;

pub struct Buffer {
    fb: &'static Framebuffer<'static>,
}

impl Buffer {
    fn new() -> Self {
        Self {
            fb: &boot_info().framebuffer,
        }
    }

    pub fn write_pixel(&self, x: u64, y: u64, color: u32) {
        if (x >= self.fb.height()) || (y >= self.fb.width()) {
            serial_println!("Buffer out of bounds: {}x{}", x, y);
            return;
        }

        let offset = (x * self.fb.pitch() + y * 4) as usize;
        unsafe {
            self.fb
                .addr()
                .add(offset)
                .cast::<u32>()
                .write_volatile(color);
        }
    }

    pub fn write_frame(&self, src: &[u32], src_w: u64, src_h: u64, dst_col: u64, dst_row: u64) {
        let fb_width = self.fb.width() as u64;
        let fb_height = self.fb.height() as u64;
        if src_w == 0 || src_h == 0 {
            return;
        }
        if dst_col + src_w > fb_width || dst_row + src_h > fb_height {
            serial_println!(
                "write_frame: out of bounds: src {}x{} -> dst {}x{} fb {}x{}",
                src_w,
                src_h,
                dst_col,
                dst_row,
                fb_width,
                fb_height
            );
            return;
        }

        let needed_len = (src_w as usize)
            .checked_mul(src_h as usize)
            .unwrap_or(usize::MAX);
        if src.len() < needed_len {
            serial_println!(
                "write_frame: src slice too small ({} < {})",
                src.len(),
                needed_len
            );
            return;
        }

        // Ensure the framebuffer's pitch can hold the source row in bytes.
        let bytes_per_src_row = (src_w as u64).checked_mul(4).unwrap_or(u64::MAX);
        if bytes_per_src_row > self.fb.pitch() {
            serial_println!(
                "write_frame: src row bytes ({}) > fb pitch ({}) — cannot copy linearly",
                bytes_per_src_row,
                self.fb.pitch()
            );
            return;
        }

        // perform per-scanline copy
        for row in 0..(src_h as usize) {
            let src_row_ptr = unsafe { src.as_ptr().add(row * src_w as usize) };
            let dst_offset = ((dst_row + row as u64) * self.fb.pitch() + dst_col * 4) as usize;
            unsafe {
                let fb_base = self.fb.addr();
                let dst_ptr = fb_base.add(dst_offset) as *mut u32;
                core::ptr::copy_nonoverlapping(src_row_ptr, dst_ptr, src_w as usize);
            }
        }
    }

    pub fn scroll_lines(&self, lines: u64) {
        let height = self.fb.height();
        if lines == 0 || lines >= height {
            self.clear_rows(0, height);
            return;
        }

        let pitch = self.fb.pitch();
        let bytes_per_row = pitch as u64;

        let src_offset = (lines * pitch) as usize;
        let dst = self.fb.addr();
        let src = unsafe { dst.add(src_offset) };
        let bytes_to_move = ((height - lines) * bytes_per_row) as usize;

        unsafe {
            core::ptr::copy(src, dst, bytes_to_move);
        }

        self.clear_rows(height - lines, lines);
    }

    pub fn clear_rows(&self, start_row: u64, count: u64) {
        let pitch = self.fb.pitch();
        let bytes_per_row = pitch as usize;
        let start_offset = (start_row * pitch) as usize;
        let total_bytes = (count * bytes_per_row as u64) as usize;

        let base = unsafe { self.fb.addr().add(start_offset) };

        unsafe {
            core::ptr::write_bytes(base, 0, total_bytes);
        }
    }
}

lazy_static! {
    pub static ref BUFFER: Buffer = Buffer::new();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}
