#![no_std]
#![no_main]

mod boot;
mod framebuffer;
pub mod serial;

use framebuffer::writer::WRITER;
use x86_64::instructions::hlt;

fn main() -> ! {
    println!("emuOS!");

    let fb = &boot::boot_info().framebuffer;
    println!("{}x{}", fb.width(), fb.height());
    println!("{}B", fb.pitch());
    println!("{}B", fb.bpp() / 8);

    for _ in 0..60 {
        print!("Hello, world!");
    }

    let mut buf = WRITER.lock();
    buf.change_color(0xFF00FF00);
    drop(buf);

    for _ in 0..60 {
        print!("Hello, Tirth!");
    }

    let mut buf = WRITER.lock();
    buf.change_color(0xFF0000FF);
    drop(buf);

    for _ in 0..60 {
        println!("emuOS");
    }

    // let width = framebuffer.width();
    // let height = framebuffer.height();

    // for i in 0..height {
    //     for j in 0..width {
    //         let color: u32 = if (i == 0 || i == height - 1) || (j == 0 || j == width - 1) {
    //             0xFFFF0000
    //         } else {
    //             0xFFFFFFFF
    //         };
    //
    //         let pixel_offset = i * framebuffer.pitch() + j * 4;
    //
    //         unsafe {
    //             framebuffer
    //                 .addr()
    //                 .add(pixel_offset as usize)
    //                 .cast::<u32>()
    //                 .write(color)
    //         };
    //     }
    // }

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
