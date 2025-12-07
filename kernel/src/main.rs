#![no_std]
#![no_main]

extern crate alloc;

use kernel::{framebuffer::screen::SCREEN, println};
use x86_64::instructions::hlt;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    println!("FPS: 0");

    let mut screen = SCREEN.lock();
    loop {
        screen.write_buffer(&[[0x00FFFFFF; 256]; 240]);
    }

   // loop { hlt(); }
}
