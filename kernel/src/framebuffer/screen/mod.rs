pub mod framework;
pub mod tv;

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
        // serial_println!("write_buffer: {} APs", parts);

        let start_col = self.width_offset as usize;
        let src_ptr = buffer as *const [[u32; 256]; 240] as usize;

        // serial_println!("Src ptr: {:#x}", src_ptr);

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
}

lazy_static! {
    pub static ref SCREEN: Mutex<Screen> = Mutex::new(Screen::new());
}
