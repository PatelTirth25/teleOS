#![no_std]
#![no_main]

extern crate alloc;

use kernel::println;
use x86_64::instructions::hlt;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    println!("emuOS! from main.rs");
    loop { hlt(); }
}
