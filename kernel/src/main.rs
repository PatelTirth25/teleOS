#![no_std]
#![no_main]

extern crate alloc;

use kernel::{framebuffer::{fps::increment_frame_count, screen::SCREEN}, println};
use x86_64::instructions::hlt;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    println!("FPS: 0");

    loop {
        let mut screen = SCREEN.lock();
        screen.write_buffer(&[[0x00FFFFFF; 256]; 240]);
        drop(screen);
        increment_frame_count();
    }

   // loop { hlt(); }
}
