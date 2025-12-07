use super::{fps::increment_frame_count, BUFFER};
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
        let start_col = self.width_offset;
        let mut scaled_buffer: [[u32; 1024]; 960] = [[0; 1024]; 960];

        // scale the buffer to 4 times
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
