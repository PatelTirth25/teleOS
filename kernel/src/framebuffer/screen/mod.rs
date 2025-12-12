pub mod framework;
pub mod tv;
use super::{fps::increment_frame_count, BUFFER};
use crate::serial_println;
use core::{hint::spin_loop, sync::atomic::Ordering};
use framework::{choose_worker_count_excluding_bsp, WORK};
use lazy_static::lazy_static;
use spin::Mutex;
pub struct Screen {
    width_offset: u64,
}
impl Screen {
    pub fn new() -> Self {
        Self { width_offset: 320 }
    }
    pub fn write_buffer(&mut self, buffer: &[[u32; 256]; 240]) {
        // choose number of workers (APs only)
        let parts = choose_worker_count_excluding_bsp();
        if parts == 0 {
            serial_println!("write_buffer: not enough APs to multithread (need >= 2 APs)");
            return;
        }
        let start_col = self.width_offset as usize;
        let src_ptr = buffer as *const [[u32; 256]; 240] as usize;
        // publish work: set pointer + params before bumping seq
        WORK.src_ptr.store(src_ptr, Ordering::Release);
        WORK.start_col.store(start_col, Ordering::Release);
        WORK.parts.store(parts, Ordering::Release);
        WORK.pending.store(parts, Ordering::Release);
        // increment sequence to notify APs (they spin on seq).
        WORK.seq.fetch_add(1, Ordering::AcqRel);
        // wait until workers finish. The last worker does final write_frame.
        while WORK.pending.load(Ordering::Acquire) != 0 {
            spin_loop();
        }
    }

    pub fn write_buffer_single(&mut self, buffer: &[[u32; 256]; 240]) {
        let start_col = self.width_offset;
        let mut scaled_buffer: [[u32; 1024]; 960] = [[0; 1024]; 960];
        for y in 0..960 {
            for x in 0..1024 {
                scaled_buffer[y][x] = buffer[y / 4][x / 4];
            }
        }
        let flat: &[u32] = unsafe {
            core::slice::from_raw_parts(scaled_buffer.as_ptr() as *const u32, 1024 * 960)
        };
        BUFFER.write_frame(flat, 1024, 960, start_col, 0);
        increment_frame_count();
    }
}
lazy_static! {
    pub static ref SCREEN: Mutex<Screen> = Mutex::new(Screen::new());
}
