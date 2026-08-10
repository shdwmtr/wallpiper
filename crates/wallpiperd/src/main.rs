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
mod wine;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use wallpiper_protocol::MonitorGeometry;

fn main() {
    if let Some(cmd) = std::env::args().nth(1) {
        if !commands::dispatch(&cmd) {
            eprintln!("unknown command: {cmd}");
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

    let monitor_state: Arc<Mutex<Option<MonitorGeometry>>> = Arc::new(Mutex::new(None));
    let patient = matches!(strategy, portal::PortalSpawnStrategy::ExternallyManaged { .. });
    portal::spawn_geometry_watcher(portal_name.clone(), patient, monitor_state.clone());

    {
        let monitor_state = monitor_state.clone();
        std::thread::spawn(move || {
            let monitor = portal::wait_for_geometry(&monitor_state);
            println!("detected monitor: {monitor:?}");
            if let Some(sel) = selection::load_selection() {
                println!("restoring last selection: {sel:?}");
                renderer::swap_renderer(&sel.file, &sel.location, monitor);
            }
        });
    }

    std::thread::spawn(|| {
        let stdin = std::io::stdin();
        for line in stdin.lines() {
            let Ok(line) = line else { break };
            let cmd = line.trim();
            if cmd.is_empty() {
                continue;
            }
            if !commands::dispatch(cmd) {
                println!("unknown command: {cmd}");
            }
        }
    });

    println!("wallpiperd ready");
    std::thread::sleep(Duration::MAX);
}
