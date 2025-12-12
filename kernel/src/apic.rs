use x86_64::instructions::port::Port;
use x86_64::registers::model_specific::Msr;

use crate::serial_println;

// APIC Timer Vector (interrupt number)
pub const APIC_TIMER_VECTOR: u8 = 32;

// IA32_APIC_BASE MSR
const IA32_APIC_BASE: u32 = 0x1B;
const APIC_BASE_ENABLE: u64 = 1 << 11;

// xAPIC MMIO offsets
const XAPIC_ID: usize = 0x020;
const XAPIC_EOI: usize = 0x0B0;
const XAPIC_SVR: usize = 0x0F0;
const XAPIC_LVT_TIMER: usize = 0x320;
const XAPIC_TIMER_INIT: usize = 0x380;
const XAPIC_TIMER_CURRENT: usize = 0x390;
const XAPIC_TIMER_DIV: usize = 0x3E0;

// Local APIC base address
const APIC_BASE_ADDRESS: usize = 0xFEE0_0000;

// Timer modes
const TIMER_PERIODIC: u32 = 1 << 17;
const TIMER_MASKED: u32 = 1 << 16;

// MMIO access functions for xAPIC mode
#[inline]
unsafe fn xapic_read(offset: usize) -> u32 {
    unsafe {
        let ptr = (APIC_BASE_ADDRESS + offset) as *const u32;
        ptr.read_volatile()
    }
}

#[inline]
unsafe fn xapic_write(offset: usize, value: u32) {
    unsafe {
        let ptr = (APIC_BASE_ADDRESS + offset) as *mut u32;
        ptr.write_volatile(value);
    }
}

/// Disable the legacy PIC timer only (keep keyboard working)
pub fn disable_pic_timer() {
    unsafe {
        let mut pic1_data = Port::<u8>::new(0x21);

        // Mask only the timer interrupt (IRQ0) on master PIC
        // Keep keyboard (IRQ1) and other interrupts unmasked
        let current_mask = pic1_data.read();
        pic1_data.write(current_mask | 0x01); // Mask IRQ0 (timer) only
    }
}

/// Initialize APIC on current core
pub fn init_apic() -> Result<(), &'static str> {
    unsafe {
        let mut apic_base_msr = Msr::new(IA32_APIC_BASE);
        let mut apic_base = apic_base_msr.read();

        // Check if APIC is available
        if apic_base & APIC_BASE_ENABLE == 0 {
            return Err("APIC not available on this CPU");
        }

        // Enable APIC if not already enabled
        if apic_base & APIC_BASE_ENABLE == 0 {
            apic_base |= APIC_BASE_ENABLE;
            apic_base_msr.write(apic_base);
        }

        // Set Spurious Interrupt Vector Register using MMIO
        // Bit 8: APIC Software Enable/Disable
        // Bits 0-7: Spurious Vector
        xapic_write(XAPIC_SVR, 0x1FF); // Vector 0xFF + enable bit
    }

    Ok(())
}

/// Calibrate APIC timer using PIT for accurate timing
pub fn calibrate_apic_timer() -> u32 {
    unsafe {
        // Set APIC timer to maximum count for calibration
        xapic_write(XAPIC_TIMER_DIV, 0x3); // Divide by 16
        xapic_write(XAPIC_LVT_TIMER, TIMER_MASKED | (APIC_TIMER_VECTOR as u32));
        xapic_write(XAPIC_TIMER_INIT, 0xFFFFFFFF); // Maximum count

        // Configure PIT for 10ms delay (100Hz)
        // PIT channel 0, mode 2 (rate generator), binary count
        let mut pit_cmd = Port::<u8>::new(0x43);
        let mut pit_data = Port::<u8>::new(0x40);

        pit_cmd.write(0x34); // Channel 0, lobyte/hibyte, mode 2
        pit_data.write(0x9B); // 0x9B27 = 1193 for 10ms at 1.19318MHz
        pit_data.write(0x2E);

        // Wait for PIT tick
        let mut current_count = 0xFFFFFFFF;
        for _ in 0..10000 {
            // Wait loop with timeout
            current_count = xapic_read(XAPIC_TIMER_CURRENT);
            if current_count < 0xFFFFFFFF {
                break;
            }
        }

        // Stop APIC timer
        xapic_write(XAPIC_TIMER_INIT, 0);

        // Calculate ticks per 10ms
        let ticks_10ms = 0xFFFFFFFF - current_count;

        // Calculate ticks per second: ticks_10ms * 100
        let ticks_per_second = ticks_10ms * 100;

        ticks_per_second
    }
}

/// Initialize APIC timer on current core
pub fn init_apic_timer() -> Result<(), &'static str> {
    unsafe {
        // Calibrate APIC timer first
        let ticks_per_second = calibrate_apic_timer();

        // Calculate initial count for 100Hz
        let initial_count = ticks_per_second;

        // Set divide configuration (divide by 16)
        xapic_write(XAPIC_TIMER_DIV, 0x3); // Divide by 16

        // Set initial count for 100Hz
        xapic_write(XAPIC_TIMER_INIT, initial_count as u32);

        // Enable timer in periodic mode
        xapic_write(XAPIC_LVT_TIMER, TIMER_PERIODIC | (APIC_TIMER_VECTOR as u32));
    }

    Ok(())
}

/// Send End of Interrupt
#[inline]
pub fn end_of_interrupt() {
    unsafe {
        // Use xAPIC MMIO
        xapic_write(XAPIC_EOI, 0);
    }
}

/// Get current APIC ID
pub fn get_apic_id() -> u32 {
    unsafe { xapic_read(XAPIC_ID) }
}

/// Check if APIC is initialized
pub fn is_apic_initialized() -> bool {
    unsafe {
        let apic_base_msr = Msr::new(IA32_APIC_BASE);
        let apic_base = apic_base_msr.read();
        (apic_base & APIC_BASE_ENABLE) != 0
    }
}
