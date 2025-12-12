#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod boot;
pub mod framebuffer;
pub mod gdt;
pub mod interrupt;
pub mod memory;
pub mod serial;
pub mod apic;
mod tests;

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

pub fn lib_main() -> ! {
    println!("emuOS! from Test");

    test_main();

    loop {
        hlt();
    }
}

fn tests() -> &'static [(&'static str, fn())] {
    use tests::framebuffer::test_println;
    use tests::framebuffer::test_screen;
    use tests::heap::test_heap_allocations;
    use tests::trivial_assertion;

    &[
        ("trivial_assertion", trivial_assertion),
        ("test_heap_allocations", test_heap_allocations),
        ("test_println", test_println),
        ("test_screen", test_screen),
    ]
}

pub fn test_main() {
    for (name, f) in tests() {
        serial_print!("{}... ", name);
        f();
        serial_println!("[ok]");
    }
    exit_qemu(QemuExitCode::Success)
}

#[cfg(not(feature = "qemu_test"))]
#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    serial_print!("{}...\n", info);
    loop {
        hlt();
    }
}

#[cfg(feature = "qemu_test")]
#[panic_handler]
fn test_panic(info: &core::panic::PanicInfo) -> ! {
    serial_print!("{}...\n", info);
    exit_qemu(QemuExitCode::Failed)
}
