#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use kernel::println;
use kernel::kernel_main;

fn main() -> ! {
    println!("emuOS!");
    kernel_main()
}
