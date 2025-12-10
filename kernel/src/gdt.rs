use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::u16;
use alloc::boxed::Box;
use lazy_static::lazy_static;
use limine::mp::Cpu;
use spin::Mutex;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

use crate::boot::boot_info;

/// tune these values as needed
pub const MAX_CPUS: usize = 9;
pub const KERNEL_STACK_SIZE: usize = 1000 * 1024; // 1000 KiB
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// Choose a reasonable size for the DF stack per CPU:
const DF_STACK_SIZE: usize = 16 * 1024; // 16 KiB (adjust if you want bigger)
const DF_STACK_ALIGN: u64 = 16; // stack alignment (ABI)

/// Static per-cpu kernel stacks (stable addresses)
#[unsafe(link_section = ".bss.stack")]
static mut KERNEL_STACKS: [[u8; KERNEL_STACK_SIZE]; MAX_CPUS] =
    [[0; KERNEL_STACK_SIZE]; MAX_CPUS];

#[unsafe(link_section = ".bss.stack")]
static mut DF_STACKS: [[u8; DF_STACK_SIZE]; MAX_CPUS] = [[0; DF_STACK_SIZE]; MAX_CPUS];
// The above weird initializer avoids long compile-time expressions in some toolchains.
// If your toolchain rejects it, replace with: `[[0u8; KERNEL_STACK_SIZE]; MAX_CPUS]` under nightly.

/// Per-cpu TSS storage (stable addresses). We'll initialize for `num_cpus` at runtime.
static mut PERCPU_TSS: [MaybeUninit<TaskStateSegment>; MAX_CPUS] =
    unsafe { MaybeUninit::<[MaybeUninit<TaskStateSegment>; MAX_CPUS]>::uninit().assume_init() };

/// The TSS selectors for each cpu will be stored here after building GDT
static mut PERCPU_TSS_SELECTORS: [SegmentSelector; MAX_CPUS] = [SegmentSelector::new(0,PrivilegeLevel::Ring0); MAX_CPUS];

// A single shared GDT and code/data selectors. The GDT itself is created on BSP
// in `init_percpu_gdt()` and then loaded. APs should call `load_gdt()` and then `load_tss_for_core()`.
lazy_static! {
    // store &'static GlobalDescriptorTable<32> so load() can be called
    static ref SHARED_GDT: Mutex<Option<(&'static GlobalDescriptorTable<32>, SegmentSelector, SegmentSelector)>> =
        Mutex::new(None);
}

/// How many CPUs we built the GDT for
static INITIALIZED_CPUS: AtomicUsize = AtomicUsize::new(0);

/// Initialize the shared GDT with per-cpu TSS descriptors.
/// Call on BSP *after* you know how many CPUs you will support (e.g. boot_info.cpus.len()).
/// This builds the GDT containing code/data + N TSS entries and loads it (lgdt).
pub fn init_percpu_gdt(num_cpus: usize) {
    assert!(num_cpus <= MAX_CPUS);

    // 1) Initialize PERCPU_TSS storage in place
    for i in 0..num_cpus {
        unsafe {
            PERCPU_TSS[i].as_mut_ptr().write(TaskStateSegment::new());
        }
    }

    // 2) Build GDT: code, data, and a TSS descriptor per cpu
    let mut gdt: GlobalDescriptorTable<32> = GlobalDescriptorTable::<32>::empty();
    let code_sel = gdt.append(Descriptor::kernel_code_segment());
    let data_sel = gdt.append(Descriptor::kernel_data_segment());

    for i in 0..num_cpus {
        let tss_ref: &'static TaskStateSegment = unsafe { &*PERCPU_TSS[i].as_ptr() };
        let tss_sel = gdt.append(Descriptor::tss_segment(tss_ref));
        unsafe { PERCPU_TSS_SELECTORS[i] = tss_sel; }
    }

    // Leak the GDT so it becomes &'static
    let gdt_box = Box::new(gdt);
    let gdt_static_ref: &'static GlobalDescriptorTable<32> = Box::leak(gdt_box);

    // Store the static ref + selectors in SHARED_GDT
    {
        let mut lock = SHARED_GDT.lock();
        *lock = Some((gdt_static_ref, code_sel, data_sel));
    }

    INITIALIZED_CPUS.store(num_cpus, Ordering::SeqCst);

    // load GDT on BSP immediately
    load_gdt();
}

/// Return how many CPUs were initialized
pub fn initialized_cpus() -> usize {
    INITIALIZED_CPUS.load(Ordering::SeqCst)
}

/// Set the kernel stack top (rsp0) for a given cpu index in its TSS.
/// Call this on BSP before waking APs (so the TSS already has the stack).
pub fn set_stack_for_cpu(cpu_index: usize, stack_top: VirtAddr) {
    assert!(cpu_index < initialized_cpus());
    unsafe {
        let tss = &mut *PERCPU_TSS[cpu_index].as_mut_ptr();
        tss.privilege_stack_table[0] = stack_top;
    }
}

/// Helper: return kernel stack top for cpu_index (VirtAddr)
pub fn kernel_stack_top(cpu_index: usize) -> VirtAddr {
    assert!(cpu_index < MAX_CPUS);
    unsafe {
        let base = KERNEL_STACKS[cpu_index].as_ptr() as u64;
        VirtAddr::new(base + KERNEL_STACK_SIZE as u64)
    }
}

/// Optionally set IST entry for cpu
pub fn set_ist_for_cpu(cpu_index: usize, ist_index: usize, ist_top: VirtAddr) {
    assert!(cpu_index < initialized_cpus());
    unsafe {
        let tss = &mut *PERCPU_TSS[cpu_index].as_mut_ptr();
        tss.interrupt_stack_table[ist_index] = ist_top;
    }
}

/// Load the shared GDT on the current core (safe to call on APs)
pub fn load_gdt() {
    use x86_64::instructions::segmentation::{CS, DS, ES, SS};

    // Call `load()` while holding the mutex so the GDT object lives for the lgdt op.
    let (code_sel, data_sel) = {
        let lock = SHARED_GDT.lock();
        let slot = lock.as_ref().expect("GDT not initialized");
        // call load() on the &'static GlobalDescriptorTable<32> we stored
        slot.0.load();
        (slot.1, slot.2)
        // lock dropped here
    };

    // Now it's safe to reload segment registers using the copied selectors.
    unsafe {
        CS::set_reg(code_sel);
        DS::set_reg(data_sel);
        ES::set_reg(data_sel);
        SS::set_reg(data_sel);
    }
}

/// Load the per-cpu TSS selector for this core (ltr).
/// `core_index` must be less than `initialized_cpus()`.
pub fn load_tss_for_core(core_index: usize) {
    assert!(core_index < initialized_cpus());
    unsafe {
        let sel = PERCPU_TSS_SELECTORS[core_index];
        load_tss(sel);
    }
}

/// Return the top-of-stack VirtAddr for the DF stack of `cpu_index`.
/// The returned address is 16-byte aligned.
pub fn df_stack_top_for(cpu_index: usize) -> VirtAddr {
    assert!(cpu_index < MAX_CPUS);
    unsafe {
        let base = DF_STACKS[cpu_index].as_ptr() as u64;
        // top before alignment
        let top = base + DF_STACK_SIZE as u64;
        // align down to 16 bytes (stack grows down)
        let aligned_top = top & !(DF_STACK_ALIGN - 1);
        VirtAddr::new(aligned_top)
    }
}

pub fn init_gdt() {
    let boot_info = boot_info();
    let num_cpus = boot_info.cpus.len();

    // 1) build per-cpu GDT and TSS storage
    crate::gdt::init_percpu_gdt(num_cpus);

    // 2) set per-cpu kernel stacks and DF ISTs BEFORE waking APs
    for i in 0..num_cpus {
        let top = crate::gdt::kernel_stack_top(i);
        crate::gdt::set_stack_for_cpu(i, top);
        let df_top = crate::gdt::df_stack_top_for(i);
        crate::gdt::set_ist_for_cpu(i, DOUBLE_FAULT_IST_INDEX as usize, df_top);
    }

    // 3) load GDT & TSS for BSP (assumed BSP index 0)
    crate::gdt::load_gdt();
    crate::gdt::load_tss_for_core(0);

    // 4) load IDT on BSP
    crate::interrupt::init_idt();

    // 5) publish stack top so trampoline (or direct entry) can pick it up on AP
    for (i, cpu) in boot_info.cpus.iter().enumerate() {
        cpu.extra.store(crate::gdt::kernel_stack_top(i).as_u64(), core::sync::atomic::Ordering::SeqCst);
    }

    // 6) Tell Limine where APs should jump to.
    //    Skip BSP (assumed index 0) so you don't jump BSP.
    for (i, cpu) in boot_info.cpus.iter().enumerate() {
        if i == 0 { continue; } // skip BSP; adapt if your BSP is not index 0
        cpu.goto_address.write(ap_main_direct); 
    }

    // 7) Now we can enable interrupts on BSP (after GDT/TSS/IDT are set)
    x86_64::instructions::interrupts::enable();

}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn ap_main_direct(cpu_ptr: &Cpu) -> ! {

    // find core index by pointer equality to boot_info.cpus entries
    let core_index = {
        let boot = boot_info(); // your function that returns the Limine boot info struct
        let mut found = None;
        for (i, c) in boot.cpus.iter().enumerate() {
            if core::ptr::eq(*c, cpu_ptr) {
                found = Some(i);
                break;
            }
        }
        match found {
            Some(i) => i,
            None => {
                loop { x86_64::instructions::hlt(); }
            }
        }
    };

    // load same shared GDT and per-core TSS
    crate::gdt::load_gdt();
    crate::gdt::load_tss_for_core(core_index);

    // load IDT on this AP as well
    crate::interrupt::init_idt();

    // Now safe to enable interrupts on this AP
    x86_64::instructions::interrupts::enable();

    crate::framebuffer::screen::framework::ap_worker_loop(core_index)

}
