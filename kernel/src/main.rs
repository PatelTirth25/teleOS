#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod boot;
mod framebuffer;
pub mod gdt;
pub mod interrupt;
pub mod memory;
pub mod serial;

use x86_64::instructions::hlt;

fn main() -> ! {
    println!("emuOS!");

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
