pub(crate) const CAPTURE_SOCKET_PATH: &str = "/tmp/wallpiper-capture.sock";
pub(crate) const LOG_PATH: &str = "/tmp/vulkan-layer-capture.log";

pub(crate) const TARGET_PROCESS_NAMES: &[&str] = &["wallpaper64.exe"];
pub(crate) const TARGET_CMDLINE_MARKERS: &[&str] = &["webwallpaper64.exe"];

pub(crate) const CAPTURE_SLOT_COUNT: usize = 3;
pub(crate) const SLOT_FENCE_TIMEOUT_NS: u64 = 4_000_000;

pub(crate) const LOG_SAMPLE_WARMUP: u64 = 5;
pub(crate) const LOG_SAMPLE_INTERVAL: u64 = 300;
