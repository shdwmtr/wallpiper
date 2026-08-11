mod audio;
mod cleanup;
mod commands;
mod config;
mod portal;
mod process;
mod renderer;
mod selection;
mod signals;
mod vk_layer;
mod wallpaper;
mod wine;

use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        if !commands::dispatch(&args) {
            eprintln!("unknown command: {}", args.join(" "));
        }
        return;
    }

    println!("wallpiperd starting");

    signals::reap_children_forever();
    signals::install_shutdown_handler(cleanup::cleanup);

    vk_layer::write_vk_layer_manifest();

    wine::ensure_persistent_wineserver();
    std::thread::sleep(Duration::from_millis(300));

    let portal_name = config::portal_name();
    println!("using portal: {portal_name}");

    let strategy = portal::portal_spawn_strategy(&portal_name);
    portal::spawn_portal(&strategy);

    let patient = matches!(
        strategy,
        portal::PortalSpawnStrategy::ExternallyManaged { .. }
    );
    portal::spawn_geometry_watcher(portal_name.clone(), patient);

    std::thread::spawn(move || {
        let monitor = portal::wait_for_geometry();
        println!("detected monitor: {monitor:?}");
        if let Some(sel) = selection::load_selection() {
            println!("restoring last selection: {sel:?}");
            renderer::swap_renderer(&sel.file, &sel.location, monitor, sel.volume, sel.muted);
        }
    });

    std::thread::spawn(|| {
        let stdin = std::io::stdin();
        for line in stdin.lines() {
            let Ok(line) = line else { break };
            let args: Vec<&str> = line.split_whitespace().collect();
            if args.is_empty() {
                continue;
            }
            if !commands::dispatch(&args) {
                println!("unknown command: {}", args.join(" "));
            }
        }
    });

    println!("wallpiperd ready");
    std::thread::sleep(Duration::MAX);
}
