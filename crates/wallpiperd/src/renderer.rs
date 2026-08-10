use std::fs::File;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use wallpiper_protocol::MonitorGeometry;

use crate::config::{self, COMPATDATA, PROTON_BIN, STEAM_ROOT, WE_EXE};
use crate::process::{find_renderer_pid, find_renderer_pids_for_tag};
use crate::vk_layer::VK_CAPTURE_LAYER_NAME;

pub fn tag_for(location: &str) -> String {
    format!("wallpiper-{location}")
}

pub fn set_paused(paused: bool) {
    let Some(pid) = find_renderer_pid() else {
        println!(
            "{}: no active renderer found",
            if paused { "pause" } else { "resume" }
        );
        return;
    };
    let sig = if paused { libc::SIGSTOP } else { libc::SIGCONT };
    let res = unsafe { libc::kill(pid, sig) };
    println!(
        "{} renderer pid={pid} -> {}",
        if paused { "paused" } else { "resumed" },
        if res == 0 { "ok" } else { "failed" }
    );
}

pub fn spawn_renderer(file: &str, location: &str, monitor: MonitorGeometry) {
    let tag = tag_for(location);
    println!(
        "spawning renderer: file={file} location={location} tag={tag} {}x{}",
        monitor.width, monitor.height
    );

    let Ok(logfile) = File::create(format!("/tmp/wallpiperd-{location}.log")) else {
        println!("failed to create renderer logfile for {location}");
        return;
    };
    let Ok(logfile_err) = logfile.try_clone() else {
        return;
    };

    let mut cmd = Command::new(PROTON_BIN);
    cmd.arg("run")
        .arg(WE_EXE)
        .arg("-control")
        .arg("openWallpaper")
        .arg("-file")
        .arg(file)
        .arg("-playInWindow")
        .arg(&tag)
        .arg("-borderless")
        .arg("-width")
        .arg(monitor.width.to_string())
        .arg("-height")
        .arg(monitor.height.to_string())
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", STEAM_ROOT)
        .env("STEAM_COMPAT_DATA_PATH", COMPATDATA)
        .env("LD_PRELOAD", config::preload_paths())
        .env("VK_ADD_LAYER_PATH", config::vk_layer_path())
        .env("VK_INSTANCE_LAYERS", VK_CAPTURE_LAYER_NAME)
        .env("WALLPIPER_MONITOR_X", monitor.x.to_string())
        .env("WALLPIPER_MONITOR_Y", monitor.y.to_string())
        .env(
            "WALLPIPER_MONITOR_LOGICAL_WIDTH",
            monitor.logical_width.to_string(),
        )
        .env(
            "WALLPIPER_MONITOR_LOGICAL_HEIGHT",
            monitor.logical_height.to_string(),
        )
        .env(
            "WALLPIPER_PORTAL_CTL_SOCKET",
            wallpiper_protocol::ctl_socket_path(&config::portal_name()),
        )
        .stdout(Stdio::from(logfile))
        .stderr(Stdio::from(logfile_err))
        .process_group(0);

    match cmd.spawn() {
        Ok(child) => println!("spawned proton wrapper pid={}", child.id()),
        Err(e) => println!("failed to spawn: {e}"),
    }
}

pub fn swap_renderer(file: &str, location: &str, monitor: MonitorGeometry) {
    let tag = tag_for(location);

    let existing = find_renderer_pids_for_tag(&tag);
    if existing.is_empty() {
        println!("no existing renderer for {tag}, cold-starting");
    } else {
        println!(
            "renderer pid(s)={existing:?} alive for {tag}, sending hot-reload control command"
        );
    }

    spawn_renderer(file, location, monitor);
}
