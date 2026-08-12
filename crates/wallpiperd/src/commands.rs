use crate::audio;
use crate::config::{self, FALLBACK_MONITOR};
use crate::portal::{current_monitor, query_monitor_once, set_debug_overlay};
use crate::renderer::{set_paused, swap_renderer};
use crate::selection::{self, save_selection, Selection};
use crate::wallpaper::{self, Source};

pub fn dispatch(args: &[&str]) -> Result<(), String> {
    let Some((&cmd, rest)) = args.split_first() else {
        return Err("empty command".to_string());
    };
    match cmd {
        "pause" => {
            set_paused(true);
            Ok(())
        }
        "resume" => {
            set_paused(false);
            Ok(())
        }
        "mute" => {
            set_mute(true);
            Ok(())
        }
        "unmute" => {
            set_mute(false);
            Ok(())
        }
        "volume" => set_volume(rest),
        "debug" => {
            set_debug_overlay(true);
            Ok(())
        }
        "nodebug" => {
            set_debug_overlay(false);
            Ok(())
        }
        "set" => set_wallpaper(rest),
        _ => Err(format!("unknown command: {cmd}")),
    }
}

fn set_mute(muted: bool) {
    audio::set_muted(muted);
    selection::update_audio_state(None, Some(muted));
}

fn set_volume(args: &[&str]) -> Result<(), String> {
    let level = args
        .first()
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| "usage: volume <0-100>".to_string())?;
    let level = level.min(100) as u8;
    audio::set_volume(level);
    selection::update_audio_state(Some(level), None);
    Ok(())
}

fn set_wallpaper(args: &[&str]) -> Result<(), String> {
    let (source, rest) = match args {
        ["--id", id, rest @ ..] => (Source::WorkshopId(id), rest),
        [] | ["--id"] => {
            return Err(
                "usage: set <file> [location]\n       set --id <workshop_id> [location]"
                    .to_string(),
            );
        }
        [path, rest @ ..] => (Source::Path(path), rest),
    };

    let file = wallpaper::resolve(source)?;
    let location = rest.first().copied().unwrap_or("default");

    config::state_file_result()?;

    let (volume, muted) = selection::load_selection()
        .map(|s| (s.volume, s.muted))
        .unwrap_or((100, false));

    let monitor = current_monitor()
        .or_else(|| query_monitor_once(&config::portal_name()))
        .unwrap_or(FALLBACK_MONITOR);
    swap_renderer(&file, location, monitor, volume, muted);
    save_selection(&Selection {
        file,
        location: location.to_string(),
        volume,
        muted,
    });
    Ok(())
}
