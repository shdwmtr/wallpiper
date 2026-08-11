use crate::audio::set_muted;
use crate::config::{self, FALLBACK_MONITOR};
use crate::portal::{current_monitor, query_monitor_once, set_debug_overlay};
use crate::renderer::{set_paused, swap_renderer};
use crate::selection::{save_selection, Selection};
use crate::wallpaper::{self, Source};

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
        "list-wallpapers" => list_wallpapers(rest),
        "list-properties" => list_properties(rest),
        "check-config" => config::describe(),
        _ => return false,
    }
    true
}

fn set_wallpaper(args: &[&str]) {
    let (source, rest) = match args {
        ["--id", id, rest @ ..] => (Source::WorkshopId(id), rest),
        [] | ["--id"] => {
            println!("usage: set <file> [location]\n       set --id <workshop_id> [location]");
            return;
        }
        [path, rest @ ..] => (Source::Path(path), rest),
    };

    let file = match wallpaper::resolve(source) {
        Ok(file) => file,
        Err(e) => {
            println!("set: {e}");
            return;
        }
    };
    let location = rest.first().copied().unwrap_or("default");

    if let Err(e) = config::state_file_result() {
        println!("set: {e}");
        return;
    }

    let monitor = current_monitor()
        .or_else(|| query_monitor_once(&config::portal_name()))
        .unwrap_or(FALLBACK_MONITOR);
    swap_renderer(&file, location, monitor);
    save_selection(&Selection {
        file,
        location: location.to_string(),
    });
}

fn list_wallpapers(args: &[&str]) {
    let json = args.contains(&"-j");

    let wallpapers = match wallpaper::list_wallpapers() {
        Ok(wallpapers) => wallpapers,
        Err(e) => return print_error(json, "list-wallpapers", &e),
    };

    if json {
        return print_json(&wallpapers);
    }

    if wallpapers.is_empty() {
        println!("no workshop wallpapers found");
        return;
    }
    for w in wallpapers {
        println!("{}  {}  ({})", w.id, w.title, w.kind);
    }
}

fn list_properties(args: &[&str]) {
    let json = args.contains(&"-j");
    let Some(&id) = args.iter().find(|&&a| a != "-j") else {
        return print_error(json, "list-properties", "usage: list-properties <workshop_id> [-j]");
    };

    let (title, properties) = match wallpaper::properties(id) {
        Ok(result) => result,
        Err(e) => return print_error(json, "list-properties", &e),
    };

    if json {
        return print_json(&serde_json::json!({
            "id": id,
            "title": title,
            "properties": properties,
        }));
    }

    println!("{title} ({id})");
    if properties.is_empty() {
        println!("  no properties");
        return;
    }
    for p in properties {
        println!("  {:<24} {:<8} {:<32} = {}", p.key, p.kind, p.text, p.value);
    }
}

fn print_json<T: serde::Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(e) => println!("failed to serialize json: {e}"),
    }
}

fn print_error(json: bool, cmd: &str, msg: &str) {
    if json {
        print_json(&serde_json::json!({ "error": msg }));
    } else {
        println!("{cmd}: {msg}");
    }
}
