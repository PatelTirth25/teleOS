use limine::framebuffer::Framebuffer;
use limine::mp::Cpu;
use limine::response::MemoryMapResponse;
use limine::BaseRevision;
use limine::request::{FramebufferRequest, HhdmRequest, MemoryMapRequest, MpRequest, RequestsEndMarker, RequestsStartMarker};
use spin::Once;
use crate::gdt::init_gdt;
use crate::memory::heap::init_heap;
use crate::memory::{self, BootInfoFrameAllocator};

#[cfg(feature = "qemu_test")]
use crate::lib_main;
#[cfg(not(feature = "qemu_test"))]
unsafe extern "C" {
    fn kernel_main() -> !;
}

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
static MP_REQUEST: MpRequest = MpRequest::new();

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
    pub cpus: &'static [&'static Cpu],
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



    // pick only 1920x1080 or high buffer, else error out that screen res not supported
    let framebuffer = {
        let mut frame_to_choose = None;
        for fb in fb_response.framebuffers() {
            if fb.height() >= 1080 && fb.width() >= 1920 {
                frame_to_choose = Some(fb);
                break;
            }
        }
        frame_to_choose.unwrap_or_else(|| panic!("No screen resolution with  1920x1080 or higher is available"))
    };


    let memory_map = MEMORY_MAP_REQUEST.get_response().expect("No memory map available");
    let cpus = MP_REQUEST.get_response().expect("No MP response from Limine").cpus();

    let boot_info = BootInfo { framebuffer,memory_map,cpus };
    BOOT_INFO.call_once(|| boot_info);

    init();


    // list all framebuffers

    // for (i, fb) in fb_response.framebuffers().enumerate() {
    //     println!(
    //         "Framebuffer {}: {}x{} | Pitch: {} | BPP: {} | Address: {:#x}",
    //         i,
    //         fb.width(),
    //         fb.height(),
    //         fb.pitch(),
    //         fb.bpp(),
    //         fb.addr() as usize
    //     );
    // }

    #[cfg(feature = "qemu_test")]
    lib_main();

    #[cfg(not(feature = "qemu_test"))]
    unsafe { kernel_main(); } 
}

pub fn init() {
    use x86_64::VirtAddr;

    let hhdm = HHDM_REQUEST.get_response().expect("no HHDM");
    let phys_offset = VirtAddr::new(hhdm.offset());

    let mut mapper = unsafe { memory::init(phys_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(boot_info().memory_map) };

    init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // Map APIC base address
    unsafe {
        memory::map_apic(&mut mapper, &mut frame_allocator).expect("Failed to map APIC");
    }

    init_gdt();

}
