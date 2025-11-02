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
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());

    let framebuffer = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("need a framebuffer")
        .framebuffers()
        .next()
        .expect("need a framebuffer");

    let boot_info = BootInfo {
        framebuffer,
    };

    BOOT_INFO.call_once(|| boot_info);

    main()
}
