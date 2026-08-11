use crate::audio::set_muted;
use crate::config::{self, FALLBACK_MONITOR};
use crate::portal::{current_monitor, query_monitor_once, set_debug_overlay};
use crate::renderer::{set_paused, swap_renderer};
use crate::selection::{save_selection, Selection};

pub fn dispatch(args: &[&str]) -> bool {
    let Some((&cmd, rest)) = args.split_first() else {
        return false;
    };
    match cmd {
        "pause" => set_paused(true),
        "resume" => set_paused(false),
        "mute" => set_muted(true),
        "unmute" => set_muted(false),
        "debug" => set_debug_overlay(true),
        "nodebug" => set_debug_overlay(false),
        "set" => set_wallpaper(rest),
        "check-config" => config::describe(),
        _ => return false,
    }
    true
}

fn set_wallpaper(args: &[&str]) {
    let Some(&file) = args.first() else {
        println!("usage: set <file> [location]");
        return;
    };
    let location = args.get(1).copied().unwrap_or("default");

    if let Err(e) = config::state_file_result() {
        println!("set: {e}");
        return;
    }

    let monitor = current_monitor()
        .or_else(|| query_monitor_once(&config::portal_name()))
        .unwrap_or(FALLBACK_MONITOR);
    swap_renderer(file, location, monitor);
    save_selection(&Selection {
        file: file.to_string(),
        location: location.to_string(),
    });
}
