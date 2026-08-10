use crate::audio::set_muted;
use crate::portal::set_debug_overlay;
use crate::renderer::set_paused;

pub fn dispatch(cmd: &str) -> bool {
    match cmd {
        "pause" => set_paused(true),
        "resume" => set_paused(false),
        "mute" => set_muted(true),
        "unmute" => set_muted(false),
        "debug" => set_debug_overlay(true),
        "nodebug" => set_debug_overlay(false),
        _ => return false,
    }
    true
}
