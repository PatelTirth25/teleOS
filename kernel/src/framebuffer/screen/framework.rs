use core::{
    hint::spin_loop,
    slice,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use crate::boot::boot_info;

use crate::framebuffer::{fps::increment_frame_count, BUFFER};

pub const SCALED_W: usize = 1024;
const SCALED_H: usize = 960;
const MAX_WORKERS: usize = 8; // clamp to 8 workers (APs only)

/// Global work descriptor (signals and parameters)
#[repr(C)]
pub struct FrameWork {
    pub seq: AtomicU64,         // sequence id changed for each frame
    pub src_ptr: AtomicUsize,   // pointer to source [[u32;256];240] as usize
    pub parts: AtomicUsize,     // number of worker parts (even, <= MAX_WORKERS)
    pub pending: AtomicUsize,   // number of workers still pending
    pub start_col: AtomicUsize, // horizontal start column offset (u64 in write_frame)
}

impl FrameWork {
    const fn new() -> Self {
        Self {
            seq: AtomicU64::new(0),
            src_ptr: AtomicUsize::new(0),
            parts: AtomicUsize::new(0),
            pending: AtomicUsize::new(0),
            start_col: AtomicUsize::new(0),
        }
    }
}

pub static WORK: FrameWork = FrameWork::new();

/// Static mutable scaled buffer.
/// SAFETY: multiple APs will write to disjoint stripes only (guaranteed by code).
/// We use `static mut` and write to it inside `unsafe` blocks.
pub static mut SCALED_BUFFER: [u32; SCALED_W * SCALED_H] = [0u32; SCALED_W * SCALED_H];

#[inline(always)]
fn scaled_buf_ptr_u32() -> *mut u32 {
    // addr_of_mut! obtains an address without creating a reference to the static.
    // cast to *mut u32 so we can index by element.
    core::ptr::addr_of_mut!(SCALED_BUFFER) as *mut u32
}

/// Choose worker count excluding BSP (BSP not participating).
/// Returns 0 if not enough APs (i.e., available_aps < 2).
pub fn choose_worker_count_excluding_bsp() -> usize {
    let cpus = boot_info().cpus.len();
    // need total >= 3 (BSP + 2 APs)
    if cpus < 3 {
        return 0;
    }
    let mut available_aps = cpus - 1; // exclude BSP
    if available_aps > MAX_WORKERS {
        available_aps = MAX_WORKERS;
    }
    // make it even
    if available_aps % 2 == 1 {
        available_aps -= 1;
    }
    if available_aps < 2 {
        return 0;
    }
    available_aps
}

/// Per-AP worker loop. Called from ap_main_direct after GDT/TSS/IDT setup.
/// `core_index` is the index inside boot_info().cpus for this CPU (BSP excluded earlier).
pub fn ap_worker_loop(core_index: usize) -> ! {
    // Assume BSP is boot_info().cpus[0]. So AP local index = core_index - 1.
    // If BSP isn't index 0 in your environment, adapt accordingly.
    let ap_local_index = core_index.checked_sub(1).unwrap_or(usize::MAX);

    // last_seen sequence to detect new frames
    let mut last_seen = WORK.seq.load(Ordering::Acquire);

    loop {
        let seq = WORK.seq.load(Ordering::Acquire);
        if seq == last_seen {
            // no new frame; low-latency spin
            spin_loop();
            continue;
        }
        last_seen = seq;

        let parts = WORK.parts.load(Ordering::Acquire);
        if parts == 0 {
            continue;
        }

        // If this AP isn't part of current work, skip
        if ap_local_index == usize::MAX || ap_local_index >= parts {
            continue;
        }

        // get source pointer (pointer validity guaranteed by writer)
        let src_usize = WORK.src_ptr.load(Ordering::Acquire);
        let start_col = WORK.start_col.load(Ordering::Acquire) as u64;

        // compute stripe rows (even split by rows)
        let stripe_h = SCALED_H / parts;
        let y0 = ap_local_index * stripe_h;
        let y1 = if ap_local_index + 1 == parts {
            SCALED_H
        } else {
            (ap_local_index + 1) * stripe_h
        };

        if src_usize == 0 {
            // malformed work; still decrement pending so writer can continue
            let _ = WORK.pending.fetch_sub(1, Ordering::AcqRel);
            continue;
        }

        // SAFELY write to only our stripe in the global scaled buffer.
        unsafe {
            // get references
            let src: &[[u32; 256]; 240] = &*(src_usize as *const [[u32; 256]; 240]);
            let out_ptr = scaled_buf_ptr_u32();

            for y in y0..y1 {
                let mut row_buffer: [u32; SCALED_W] = [0; SCALED_W];

                for x in 0..SCALED_W {
                    let pixel = src[y / 4][x / 4];
                    row_buffer[x] = pixel;
                }

                let out_row_offset = y * SCALED_W;
                core::ptr::copy_nonoverlapping(
                    row_buffer.as_ptr(),
                    out_ptr.add(out_row_offset),
                    SCALED_W,
                );
            }
        } // end unsafe stripe write

        // mark this worker done
        let prev = WORK.pending.fetch_sub(1, Ordering::AcqRel);

        // If we are the last worker, perform the final flush to the hardware buffer.
        if prev == 1 {
            unsafe {
                // SCALED_BUFFER now fully written by all APs; pass it to BUFFER.
                // make a slice &SCALED_BUFFER
                let out_ptr = scaled_buf_ptr_u32() as *const u32;
                let out_slice: &[u32] = slice::from_raw_parts(out_ptr, SCALED_W * SCALED_H);
                BUFFER.write_frame(out_slice, SCALED_W as u64, SCALED_H as u64, start_col, 0);
                increment_frame_count();
            }
        }
    }
}
