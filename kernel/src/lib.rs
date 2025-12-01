#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod boot;
pub mod framebuffer;
pub mod gdt;
pub mod interrupt;
pub mod memory;
pub mod serial;

use x86_64::instructions::hlt;
use x86_64::instructions::port::Port;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(code: QemuExitCode) -> ! {
    unsafe {
        let mut port: Port<u32> = Port::new(0xF4);
        port.write(code as u32);
    }
    loop {
        hlt();
    }
}

pub fn kernel_main() -> ! {
    println!("emuOS!");

    #[cfg(feature = "qemu_test")]
    test_main();

    loop {
        hlt();
    }
}

#[cfg(feature = "qemu_test")]
fn tests() -> &'static [(&'static str, fn())] {
    &[("trivial_assertion", trivial_assertion)]
}

#[cfg(feature = "qemu_test")]
pub fn test_main() {
    for (name, f) in tests() {
        serial_print!("{}... ", name);
        f();
        serial_println!("[ok]");
    }
    exit_qemu(QemuExitCode::Success)
}

#[cfg(feature = "qemu_test")]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[cfg(not(feature = "qemu_test"))]
#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        hlt();
    }
}

#[cfg(feature = "qemu_test")]
#[panic_handler]
fn test_panic(_info: &core::panic::PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Failed)
}
