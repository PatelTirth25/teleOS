use crate::serial_println;
use alloc::{fmt::format, format, string::ToString};
use core::sync::atomic::{AtomicU32, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

use super::writer::WRITER;

static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
static LAST_FPS: AtomicU32 = AtomicU32::new(0);
static TICKS: AtomicU32 = AtomicU32::new(0);

pub struct FpsCounter {
    last_print_ticks: u32,
}

impl FpsCounter {
    pub const fn new() -> Self {
        Self {
            last_print_ticks: 0,
        }
    }

    pub fn tick(&mut self) {
        let ticks = TICKS.fetch_add(1, Ordering::Relaxed) + 1;

        if ticks - self.last_print_ticks >= 100 {
            let frames = FRAME_COUNT.swap(0, Ordering::Relaxed);
            LAST_FPS.store(frames, Ordering::Relaxed);
            self.last_print_ticks = ticks;

            // Print FPS to the screen
            let mut writer = WRITER.lock();

            writer.write_str_at(&frames.to_string(), 0, 5);
        }
    }

    pub fn get_fps() -> u32 {
        LAST_FPS.load(Ordering::Relaxed)
    }
}

lazy_static! {
    pub static ref FPS_COUNTER: Mutex<FpsCounter> = Mutex::new(FpsCounter::new());
}

pub fn increment_frame_count() {
    FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
}
