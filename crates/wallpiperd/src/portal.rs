use std::fs::File;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use wallpiper_protocol::{CtlRequest, CtlResponse, MonitorGeometry};

use crate::config::{self, FALLBACK_MONITOR, REPO_ROOT};

static DISPLAY_PID: Mutex<Option<i32>> = Mutex::new(None);

pub fn take_display_pid() -> Option<i32> {
    DISPLAY_PID.lock().unwrap().take()
}

pub enum PortalSpawnStrategy {
    Spawn { name: String, binary: String },
    ExternallyManaged { name: String },
}

pub fn portal_spawn_strategy(name: &str) -> PortalSpawnStrategy {
    match name {
        "kde" => PortalSpawnStrategy::ExternallyManaged {
            name: name.to_string(),
        },
        _ => PortalSpawnStrategy::Spawn {
            name: name.to_string(),
            binary: format!("{REPO_ROOT}/target/release/wallpiper-portal-{name}"),
        },
    }
}

pub fn spawn_portal(strategy: &PortalSpawnStrategy) {
    let (name, bin) = match strategy {
        PortalSpawnStrategy::Spawn { name, binary } => (name, binary),
        PortalSpawnStrategy::ExternallyManaged { name } => {
            println!(
                "portal {name} is externally managed, not spawning — waiting for its ctl socket to appear"
            );
            return;
        }
    };

    let Ok(logfile) = File::create(format!("/tmp/wallpiperd-portal-{name}.log")) else {
        println!("failed to create portal logfile");
        return;
    };
    let Ok(logfile_err) = logfile.try_clone() else {
        return;
    };
    match Command::new(bin)
        .stdout(Stdio::from(logfile))
        .stderr(Stdio::from(logfile_err))
        .process_group(0)
        .spawn()
    {
        Ok(child) => {
            println!("spawned portal ({bin}) pid={}", child.id());
            *DISPLAY_PID.lock().unwrap() = Some(child.id() as i32);
        }
        Err(e) => println!("failed to spawn portal ({bin}): {e}"),
    }
}

pub fn spawn_geometry_watcher(
    name: String,
    patient: bool,
    state: Arc<Mutex<Option<MonitorGeometry>>>,
) {
    std::thread::spawn(move || {
        let interval = if patient {
            Duration::from_secs(2)
        } else {
            Duration::from_millis(300)
        };
        let mut attempt: u32 = 0;
        let mut was_reachable = false;
        loop {
            attempt += 1;
            match wallpiper_protocol::send_ctl_request(&name, CtlRequest::Geometry) {
                Some(CtlResponse::Geometry(geometry)) => {
                    let mut guard = state.lock().unwrap();
                    if *guard != Some(geometry) {
                        println!("portal {name} geometry: {geometry:?}");
                    }
                    *guard = Some(geometry);
                    was_reachable = true;
                }
                _ => {
                    if was_reachable {
                        println!(
                            "portal {name} ctl socket unreachable, keeping last known geometry"
                        );
                        was_reachable = false;
                    }
                    if !patient && attempt >= 10 && state.lock().unwrap().is_none() {
                        println!("portal {name} never answered a geometry request, falling back to default");
                        *state.lock().unwrap() = Some(FALLBACK_MONITOR);
                    }
                }
            }
            std::thread::sleep(interval);
        }
    });
}

pub fn wait_for_geometry(state: &Arc<Mutex<Option<MonitorGeometry>>>) -> MonitorGeometry {
    loop {
        if let Some(geometry) = *state.lock().unwrap() {
            return geometry;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

pub fn detach_display() -> bool {
    matches!(
        wallpiper_protocol::send_ctl_request(&config::portal_name(), CtlRequest::Detach),
        Some(CtlResponse::Ok)
    )
}

pub fn set_debug_overlay(enabled: bool) {
    let response = wallpiper_protocol::send_ctl_request(
        &config::portal_name(),
        CtlRequest::SetDebug(enabled),
    );
    let ok = matches!(response, Some(CtlResponse::Ok));
    println!(
        "debug overlay {} -> {}",
        if enabled { "on" } else { "off" },
        if ok { "ok" } else { "failed" }
    );
}
