use limine::framebuffer::Framebuffer;
use limine::response::MemoryMapResponse;
use limine::BaseRevision;
use limine::request::{FramebufferRequest, HhdmRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker};
use spin::Once;
use crate::memory::heap::init_heap;
use crate::memory::{self, BootInfoFrameAllocator};
use crate::{gdt, interrupt, kernel_main};

/// Sets the base revision to the latest revision supported by the crate.
/// See specification for further info.
/// Be sure to mark all limine requests with #[used], otherwise they may be removed by the compiler.
#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Define the start and end markers for Limine requests.
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

pub struct BootInfo {
    pub framebuffer: Framebuffer<'static>,
    pub memory_map: &'static MemoryMapResponse,
}

static BOOT_INFO: Once<BootInfo> = Once::new();

pub fn boot_info() -> &'static BootInfo {
    unsafe { BOOT_INFO.get().unwrap_unchecked() }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());

    let fb_response = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("No framebuffer response from Limine");

    // list all framebuffers

    // for (i, fb) in fb_response.framebuffers().enumerate() {
    //     serial_println!(
    //         "Framebuffer {}: {}x{} | Pitch: {} | BPP: {} | Address: {:#x}",
    //         i,
    //         fb.width(),
    //         fb.height(),
    //         fb.pitch(),
    //         fb.bpp(),
    //         fb.addr() as usize
    //     );
    // }

    // pick the first framebuffer for now
    let framebuffer = fb_response
        .framebuffers()
        .next()
        .expect("No framebuffer available");

    let memory_map = MEMORY_MAP_REQUEST.get_response().expect("No memory map available");

    let boot_info = BootInfo { framebuffer,memory_map };
    BOOT_INFO.call_once(|| boot_info);
    init();

    kernel_main()

}

pub fn init() {
    use x86_64::VirtAddr;
    gdt::init();
    interrupt::init_idt();

    unsafe {
        let mut pics = interrupt::PICS.lock();
        pics.initialize();
        pics.write_masks(0, 0);
    }
    x86_64::instructions::interrupts::enable();

    let hhdm = HHDM_REQUEST.get_response().expect("no HHDM");
    let phys_offset = VirtAddr::new(hhdm.offset());

    let mut mapper = unsafe { memory::init(phys_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(boot_info().memory_map) };

    init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
}
