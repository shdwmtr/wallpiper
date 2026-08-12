pub mod config;
pub mod wallpaper;

mod audio;
mod cleanup;
mod commands;
mod portal;
mod process;
mod renderer;
mod selection;
mod signals;
mod vk_layer;

use std::time::Duration;

pub fn run() {
    println!("wallpiperd starting");

    signals::reap_children_forever();
    signals::install_shutdown_handler(cleanup::cleanup);

    vk_layer::write_vk_layer_manifest();

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

    wallpiper_protocol::spawn_daemon_ctl_listener(commands::dispatch);

    println!("wallpiperd ready");
    std::thread::sleep(Duration::MAX);
}
