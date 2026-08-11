use std::fs::File;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use wallpiper_protocol::{CtlRequest, CtlResponse, MonitorGeometry};

use crate::config::{self, FALLBACK_MONITOR};

static DISPLAY_PID: Mutex<Option<i32>> = Mutex::new(None);
static MONITOR_STATE: Mutex<Option<MonitorGeometry>> = Mutex::new(None);

fn die_with_parent() -> std::io::Result<()> {
    if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    if unsafe { libc::getppid() } == 1 {
        unsafe { libc::raise(libc::SIGKILL) };
    }
    Ok(())
}

pub fn take_display_pid() -> Option<i32> {
    DISPLAY_PID.lock().unwrap().take()
}

pub fn current_monitor() -> Option<MonitorGeometry> {
    *MONITOR_STATE.lock().unwrap()
}

pub fn query_monitor_once(name: &str) -> Option<MonitorGeometry> {
    match wallpiper_protocol::send_ctl_request(name, CtlRequest::Geometry) {
        Some(CtlResponse::Geometry(geometry)) => Some(geometry),
        _ => None,
    }
}

pub enum PortalSpawnStrategy {
    Spawn { name: String, binary: String },
    ExternallyManaged { name: String },
}

pub fn portal_spawn_strategy(name: &str) -> PortalSpawnStrategy {
    match name {
        "kde" | "gnome" => PortalSpawnStrategy::ExternallyManaged {
            name: name.to_string(),
        },
        _ => PortalSpawnStrategy::Spawn {
            name: name.to_string(),
            binary: format!(
                "{}/wallpiper-portal-{name}",
                config::install_dir().display()
            ),
        },
    }
}

pub fn spawn_portal(strategy: &PortalSpawnStrategy) {
    let (name, bin) = match strategy {
        PortalSpawnStrategy::Spawn { name, binary } => (name, binary),
        PortalSpawnStrategy::ExternallyManaged { name } => {
            println!(
                "portal {name} is externally managed, not spawning: waiting for its ctl socket to appear"
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
    let mut cmd = Command::new(bin);
    cmd.stdout(Stdio::from(logfile))
        .stderr(Stdio::from(logfile_err))
        .process_group(0);
    unsafe {
        cmd.pre_exec(die_with_parent);
    }

    match cmd.spawn() {
        Ok(child) => {
            println!("spawned portal ({bin}) pid={}", child.id());
            *DISPLAY_PID.lock().unwrap() = Some(child.id() as i32);
        }
        Err(e) => println!("failed to spawn portal ({bin}): {e}"),
    }
}

pub fn spawn_geometry_watcher(name: String, patient: bool) {
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
                    let mut guard = MONITOR_STATE.lock().unwrap();
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
                    if !patient && attempt >= 10 && MONITOR_STATE.lock().unwrap().is_none() {
                        println!("portal {name} never answered a geometry request, falling back to default");
                        *MONITOR_STATE.lock().unwrap() = Some(FALLBACK_MONITOR);
                    }
                }
            }
            std::thread::sleep(interval);
        }
    });
}

pub fn wait_for_geometry() -> MonitorGeometry {
    loop {
        if let Some(geometry) = current_monitor() {
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
    let response =
        wallpiper_protocol::send_ctl_request(&config::portal_name(), CtlRequest::SetDebug(enabled));
    let ok = matches!(response, Some(CtlResponse::Ok));
    println!(
        "debug overlay {} -> {}",
        if enabled { "on" } else { "off" },
        if ok { "ok" } else { "failed" }
    );
}
