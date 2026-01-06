# Rust OS Kernel (WIP)

A basic hobby OS / kernel written in **Rust**, using the **Limine** bootloader.

## Features
- Custom separate framebuffer with fixed resolution **256×240**
- Multithreading (SMP support) for custom framebuffer
- FPS counter
- Keyboard input support
- Runs correctly on **1920×1080 or higher** displays
- Clean low-level Rust (`no_std`)

## Bootloader
- Uses **Limine** (BIOS + UEFI)

## Build & Run
Make sure you have the required toolchain and QEMU installed.

```sh
make run
