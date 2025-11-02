#![no_std]
#![no_main]

mod boot;

use boot::boot_info;
use x86_64::instructions::hlt;

fn main() -> ! {
    let boot_info = boot_info();
    let framebuffer = &boot_info.framebuffer;
    let width = framebuffer.width();
    let height = framebuffer.height();

    for i in 0..height {
        for j in 0..width {
            let color: u32 = if (i == 0 || i == height - 1) || (j == 0 || j == width - 1) {
                0xFFFF0000
            } else {
                0xFFFFFFFF
            };

            let pixel_offset = i * framebuffer.pitch() + j * 4;

            unsafe {
                framebuffer
                    .addr()
                    .add(pixel_offset as usize)
                    .cast::<u32>()
                    .write(color)
            };
        }
    }

    loop {
        hlt();
    }
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        hlt();
    }
}
