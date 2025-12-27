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

    // loop { 
    //     x86_64::instructions::hlt(); 
    // }
}

// 3 methods to write buffer on screen without overflowing stack:

// 1)
// static BUF: [[u32; 256]; 240] = [[0x00FFFFFF; 256]; 240];
// let mut screen = SCREEN.lock();
// screen.write_buffer(&BUF);

// 2)
// let mut v: Vec<u32> = Vec::with_capacity(240*256);
// for _ in 0..(240*256) { v.push(0x00FFFFFF); }
// let boxed_slice = v.into_boxed_slice(); 
// let arr_ref: &'static [[u32;256];240] = unsafe {
//     &*(boxed_slice.as_ptr() as *const [[u32;256];240])
// };
// let mut screen = SCREEN.lock();
// loop {
//     screen.write_buffer(arr_ref);
// }

// 3)
// let mut boxed_uninit = Box::<[[u32;256];240]>::new_uninit(); // heap alloc
// let ptr = boxed_uninit.as_mut_ptr() as *mut [u32;256];
// for r in 0..240 {
//     let row_ptr = unsafe { (ptr.add(r)) as *mut u32 };
//     for c in 0..256 {
//         unsafe { core::ptr::write(row_ptr.add(c), 0x00FFFFFF) };
//     }
// }
// let boxed = unsafe { boxed_uninit.assume_init() };
// let buf_static = Box::leak(boxed);
// let mut screen = SCREEN.lock();
// loop {
//     screen.write_buffer(buf_static);
// }
