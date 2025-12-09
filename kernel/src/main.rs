#![no_std]
#![no_main]

extern crate alloc;

use kernel::{framebuffer::screen::{tv, SCREEN}, println};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    println!("FPS: 0");

    let mut screen = SCREEN.lock();
    loop {
        screen.write_buffer(&tv::BUFFER1);
        screen.write_buffer(&tv::BUFFER2);
        screen.write_buffer(&tv::BUFFER3);
        screen.write_buffer(&tv::BUFFER4);
    }

    loop { 
        x86_64::instructions::hlt(); 
    }
}
