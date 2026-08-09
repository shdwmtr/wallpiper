use std::sync::LazyLock;

use crate::config::{TARGET_CMDLINE_MARKERS, TARGET_PROCESS_NAMES};

static IS_TARGET_PROCESS: LazyLock<bool> = LazyLock::new(detect_target_process);

pub(crate) fn is_target_process() -> bool {
    *IS_TARGET_PROCESS
}

fn detect_target_process() -> bool {
    let by_comm = std::fs::read_to_string("/proc/self/comm")
        .is_ok_and(|name| TARGET_PROCESS_NAMES.contains(&name.trim()));
    if by_comm {
        return true;
    }
    std::fs::read_to_string("/proc/self/cmdline").is_ok_and(|cmdline| {
        TARGET_CMDLINE_MARKERS
            .iter()
            .any(|marker| cmdline.contains(marker))
    })
}
