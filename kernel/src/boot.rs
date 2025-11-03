use limine::framebuffer::Framebuffer;
use limine::BaseRevision;
use limine::request::{FramebufferRequest, RequestsEndMarker, RequestsStartMarker};
use spin::Once;

use crate::main;

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

/// Define the start and end markers for Limine requests.
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

pub struct BootInfo {
    pub framebuffer: Framebuffer<'static>,
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

    let boot_info = BootInfo { framebuffer };
    BOOT_INFO.call_once(|| boot_info);

    main()

}
