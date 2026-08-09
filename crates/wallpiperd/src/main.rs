use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use wallpiper_protocol::{CtlRequest, CtlResponse, MonitorGeometry};

mod tray;

const SELECTION_SOCKET_NAME: &[u8] = b"wallpiper-mitm";
const STEAM_ROOT: &str = "/home/shdw/.local/share/Steam";
const PROTON_BIN: &str = "/home/shdw/.local/share/Steam/compatibilitytools.d/GE-Proton11-3/proton";
const WE_EXE: &str =
    "/home/shdw/.local/share/Steam/steamapps/common/wallpaper_engine/wallpaper64.exe";
const COMPATDATA: &str = "/home/shdw/.local/share/Steam/steamapps/compatdata/431960";
const REPO_ROOT: &str = "/home/shdw/Development/wallpiper";
const STATE_DIR: &str = "/home/shdw/.local/share/wallpiper";
const STATE_FILE: &str = "/home/shdw/.local/share/wallpiper/last_selection.json";

static DISPLAY_PID: Mutex<Option<i32>> = Mutex::new(None);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct Selection {
    file: String,
    location: String,
}

fn save_selection(sel: &Selection) {
    let _ = std::fs::create_dir_all(STATE_DIR);
    if let Ok(json) = serde_json::to_string(sel) {
        if let Err(e) = std::fs::write(STATE_FILE, json) {
            println!("failed to save selection state: {e}");
        }
    }
}

fn load_selection() -> Option<Selection> {
    let data = std::fs::read_to_string(STATE_FILE).ok()?;
    serde_json::from_str(&data).ok()
}

fn extract_json(line: &str) -> Option<Selection> {
    let idx = line.find('{')?;
    serde_json::from_str(&line[idx..]).ok()
}

fn tag_for(location: &str) -> String {
    format!("wallpiper-{location}")
}

fn pid_alive(pid: i32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

fn kill_pids_gracefully(pids: &[i32]) {
    for &pid in pids {
        let res = unsafe { libc::kill(pid, libc::SIGTERM) };
        println!(
            "SIGTERM pid={pid} -> {}",
            if res == 0 { "ok" } else { "failed" }
        );
    }
    for _ in 0..30 {
        if !pids.iter().any(|&pid| pid_alive(pid)) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    for &pid in pids {
        if pid_alive(pid) {
            let res = unsafe { libc::kill(pid, libc::SIGKILL) };
            println!(
                "pid={pid} still alive after SIGTERM grace period, SIGKILL -> {}",
                if res == 0 { "ok" } else { "failed" }
            );
        }
    }
}

fn find_renderer_pids() -> Vec<i32> {
    let mut pids = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return pids;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<i32>().ok())
        else {
            continue;
        };
        let Ok(comm) = std::fs::read_to_string(format!("/proc/{pid}/comm")) else {
            continue;
        };
        if comm.trim() == "wallpaper64.exe" {
            pids.push(pid);
        }
    }
    pids
}

fn find_renderer_pid() -> Option<i32> {
    find_renderer_pids().into_iter().next()
}

fn find_pids_for_tag(tag: &str) -> Vec<i32> {
    let marker = format!("-playInWindow {tag}");
    let mut pids = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return pids;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<i32>().ok())
        else {
            continue;
        };
        let Ok(cmdline) = std::fs::read_to_string(format!("/proc/{pid}/cmdline")) else {
            continue;
        };
        let cmdline = cmdline.replace('\0', " ");
        if cmdline.contains(&marker) {
            pids.push(pid);
        }
    }
    pids
}

fn find_renderer_pids_for_tag(tag: &str) -> Vec<i32> {
    find_pids_for_tag(tag)
        .into_iter()
        .filter(|&pid| {
            std::fs::read_to_string(format!("/proc/{pid}/comm"))
                .map(|comm| comm.trim() == "wallpaper64.exe")
                .unwrap_or(false)
        })
        .collect()
}

fn find_webwallpaper_pids() -> Vec<i32> {
    let mut pids = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return pids;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<i32>().ok())
        else {
            continue;
        };
        let Ok(cmdline) = std::fs::read_to_string(format!("/proc/{pid}/cmdline")) else {
            continue;
        };
        if cmdline.contains("webwallpaper64.exe") {
            pids.push(pid);
        }
    }
    pids
}

fn detach_display() -> bool {
    matches!(
        wallpiper_protocol::send_ctl_request(&portal_name(), CtlRequest::Detach),
        Some(CtlResponse::Ok)
    )
}

fn set_debug_overlay(enabled: bool) {
    let response = wallpiper_protocol::send_ctl_request(&portal_name(), CtlRequest::SetDebug(enabled));
    let ok = matches!(response, Some(CtlResponse::Ok));
    println!(
        "debug overlay {} -> {}",
        if enabled { "on" } else { "off" },
        if ok { "ok" } else { "failed" }
    );
}

pub(crate) fn cleanup() {
    println!("wallpiperd shutting down, cleaning up spawned processes");

    let detached = detach_display();
    println!(
        "display detach handshake -> {}",
        if detached {
            "ok"
        } else {
            "failed or timed out, proceeding anyway"
        }
    );

    let mut pids = find_renderer_pids();
    pids.extend(find_webwallpaper_pids());
    kill_pids_gracefully(&pids);

    if let Some(pid) = DISPLAY_PID.lock().unwrap().take() {
        kill_pids_gracefully(&[pid]);
    }
}

pub(crate) fn set_paused(paused: bool) {
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

fn find_renderer_sink_input(pid: i32) -> Option<String> {
    let output = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut current_id: Option<String> = None;
    let pid_marker = format!("\"{pid}\"");
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Sink Input #") {
            current_id = Some(rest.trim().to_string());
        } else if line.starts_with("application.process.id") && line.contains(&pid_marker) {
            return current_id;
        }
    }
    None
}

pub(crate) fn set_muted(muted: bool) {
    let Some(pid) = find_renderer_pid() else {
        println!("mute: no active renderer found");
        return;
    };
    let Some(sink_input_id) = find_renderer_sink_input(pid) else {
        println!("mute: no audio stream found for renderer pid={pid}");
        return;
    };
    let val = if muted { "1" } else { "0" };
    let status = Command::new("pactl")
        .args(["set-sink-input-mute", &sink_input_id, val])
        .status();
    println!("set-sink-input-mute {sink_input_id} {val} -> {status:?}");
}

fn portal_name() -> String {
    std::env::var("WALLPIPER_PORTAL").unwrap_or_else(|_| {
        panic!(
            "WALLPIPER_PORTAL not set — export WALLPIPER_PORTAL=<name> naming an installed \
             wallpiper-portal-<name> binary (e.g. WALLPIPER_PORTAL=hyprland)"
        )
    })
}

fn spawn_portal(name: &str) {
    let bin = format!("{REPO_ROOT}/target/release/wallpiper-portal-{name}");
    let Ok(logfile) = File::create(format!("/tmp/wallpiperd-portal-{name}.log")) else {
        println!("failed to create portal logfile");
        return;
    };
    let Ok(logfile_err) = logfile.try_clone() else {
        return;
    };
    match Command::new(&bin)
        .stdout(Stdio::from(logfile))
        .stderr(Stdio::from(logfile_err))
        .spawn()
    {
        Ok(child) => {
            println!("spawned portal ({bin}) pid={}", child.id());
            *DISPLAY_PID.lock().unwrap() = Some(child.id() as i32);
        }
        Err(e) => println!("failed to spawn portal ({bin}): {e}"),
    }
}

fn request_geometry(name: &str) -> MonitorGeometry {
    for attempt in 1..=10 {
        if let Some(CtlResponse::Geometry(geometry)) =
            wallpiper_protocol::send_ctl_request(name, CtlRequest::Geometry)
        {
            return geometry;
        }
        println!("portal {name} ctl socket not ready yet (attempt {attempt}), retrying");
        std::thread::sleep(Duration::from_millis(300));
    }
    println!("portal {name} never answered a geometry request, falling back to 1920x1080 at 0,0");
    MonitorGeometry {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        logical_width: 1920,
        logical_height: 1080,
    }
}

fn preload_paths() -> String {
    format!("{REPO_ROOT}/target/release/libwallpiper-preload.so")
}

const VK_CAPTURE_LAYER_NAME: &str = "VK_LAYER_wallpiper_capture";

fn vk_layer_path() -> String {
    STATE_DIR.to_string()
}

fn write_vk_layer_manifest() {
    let _ = std::fs::create_dir_all(STATE_DIR);
    let library_path = format!("{REPO_ROOT}/target/release/libVkLayer_wallpiper_capture.so");
    let manifest = format!(
        r#"{{
    "file_format_version" : "1.0.0",
    "layer" : {{
        "name": "{VK_CAPTURE_LAYER_NAME}",
        "type": "GLOBAL",
        "library_path": "{library_path}",
        "api_version": "1.1.0",
        "implementation_version": "1",
        "description": "Wallpiper frame capture layer"
    }}
}}
"#
    );
    let path = format!("{STATE_DIR}/{VK_CAPTURE_LAYER_NAME}.json");
    if let Err(e) = std::fs::write(&path, manifest) {
        println!("failed to write vk layer manifest at {path}: {e}");
    }
}

fn wine_prefix() -> String {
    format!("{COMPATDATA}/pfx")
}

fn wineserver_bin() -> String {
    let dir = std::path::Path::new(PROTON_BIN)
        .parent()
        .expect("PROTON_BIN has no parent directory");
    format!("{}/files/bin/wineserver", dir.display())
}

fn ensure_persistent_wineserver() {
    let bin = wineserver_bin();
    let prefix = wine_prefix();
    let Ok(logfile) = File::create("/tmp/wallpiperd-wineserver.log") else {
        println!("failed to create wineserver logfile");
        return;
    };
    let Ok(logfile_err) = logfile.try_clone() else {
        return;
    };
    match Command::new(&bin)
        .arg("-p")
        .env("WINEPREFIX", &prefix)
        .stdout(Stdio::from(logfile))
        .stderr(Stdio::from(logfile_err))
        .spawn()
    {
        Ok(child) => println!(
            "persistent wineserver ensured for {prefix} pid={}",
            child.id()
        ),
        Err(e) => println!("failed to start persistent wineserver ({bin}): {e}"),
    }
}

pub(crate) fn launch_ui(extra_arg: Option<&str>) {
    let Ok(logfile) = File::create("/tmp/wallpiperd-ui.log") else {
        println!("failed to create UI logfile");
        return;
    };
    let Ok(logfile_err) = logfile.try_clone() else {
        return;
    };
    let mut cmd = Command::new(PROTON_BIN);
    cmd.arg("run").arg(WE_EXE);
    if let Some(arg) = extra_arg {
        cmd.arg(arg);
    }
    let spawned = cmd
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", STEAM_ROOT)
        .env("STEAM_COMPAT_DATA_PATH", COMPATDATA)
        .env("LD_PRELOAD", preload_paths())
        .env("VK_ADD_LAYER_PATH", vk_layer_path())
        .env("VK_INSTANCE_LAYERS", VK_CAPTURE_LAYER_NAME)
        .stdout(Stdio::from(logfile))
        .stderr(Stdio::from(logfile_err))
        .spawn();
    match spawned {
        Ok(child) => println!("launched WE UI (with we-mitm tap) pid={}", child.id()),
        Err(e) => println!("failed to launch WE UI: {e}"),
    }
}

fn browse_ui_open() -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<i32>().ok())
        else {
            continue;
        };
        let Ok(comm) = std::fs::read_to_string(format!("/proc/{pid}/comm")) else {
            continue;
        };
        if comm.trim() != "wallpaperui.exe" {
            continue;
        }
        let Ok(cmdline) = std::fs::read_to_string(format!("/proc/{pid}/cmdline")) else {
            continue;
        };
        if !cmdline.contains("--type=") {
            return true;
        }
    }
    false
}

fn spawn_renderer(file: &str, location: &str, monitor: MonitorGeometry) {
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
        .env("LD_PRELOAD", preload_paths())
        .env("VK_ADD_LAYER_PATH", vk_layer_path())
        .env("VK_INSTANCE_LAYERS", VK_CAPTURE_LAYER_NAME)
        .env("WALLPIPER_MONITOR_X", monitor.x.to_string())
        .env("WALLPIPER_MONITOR_Y", monitor.y.to_string())
        .env("WALLPIPER_MONITOR_LOGICAL_WIDTH", monitor.logical_width.to_string())
        .env("WALLPIPER_MONITOR_LOGICAL_HEIGHT", monitor.logical_height.to_string())
        .env("WALLPIPER_PORTAL_CTL_SOCKET", wallpiper_protocol::ctl_socket_path(&portal_name()))
        .stdout(Stdio::from(logfile))
        .stderr(Stdio::from(logfile_err));

    match cmd.spawn() {
        Ok(child) => println!("spawned proton wrapper pid={}", child.id()),
        Err(e) => println!("failed to spawn: {e}"),
    }
}

fn swap_renderer(file: &str, location: &str, monitor: MonitorGeometry) {
    let tag = tag_for(location);

    let existing = find_renderer_pids_for_tag(&tag);
    if existing.is_empty() {
        println!("no existing renderer for {tag}, cold-starting");
    } else {
        println!("renderer pid(s)={existing:?} alive for {tag}, sending hot-reload control command");
    }

    spawn_renderer(file, location, monitor);
}

fn main() {
    if let Some(cmd) = std::env::args().nth(1) {
        match cmd.as_str() {
            "pause" => set_paused(true),
            "resume" => set_paused(false),
            "mute" => set_muted(true),
            "unmute" => set_muted(false),
            "debug" => set_debug_overlay(true),
            "nodebug" => set_debug_overlay(false),
            other => eprintln!("unknown command: {other}"),
        }
        return;
    }

    println!("wallpiperd starting");

    let selection_addr = SocketAddr::from_abstract_name(SELECTION_SOCKET_NAME)
        .expect("failed to construct selection socket address");
    let selection_socket = UnixDatagram::bind_addr(&selection_addr)
        .expect("failed to bind selection socket (is another wallpiperd already running?)");
    println!("listening for selections on abstract socket");

    {
        let mut signals = signal_hook::iterator::Signals::new([
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGTERM,
        ])
        .expect("failed to register signal handler");
        std::thread::spawn(move || {
            if signals.forever().next().is_some() {
                cleanup();
                std::process::exit(0);
            }
        });
    }

    write_vk_layer_manifest();

    ensure_persistent_wineserver();
    std::thread::sleep(Duration::from_millis(300));

    let portal = portal_name();
    println!("using portal: {portal}");

    spawn_portal(&portal);
    let monitor = request_geometry(&portal);
    println!("detected monitor: {monitor:?}");

    std::thread::sleep(Duration::from_millis(500));

    if let Some(sel) = load_selection() {
        println!("restoring last selection: {sel:?}");
        swap_renderer(&sel.file, &sel.location, monitor);
    }

    use ksni::blocking::TrayMethods;
    let _tray_handle = match (tray::WallpiperTray {
        paused: false,
        muted: false,
    })
    .spawn()
    {
        Ok(handle) => {
            println!("tray icon spawned");
            Some(handle)
        }
        Err(e) => {
            println!("failed to spawn tray icon: {e:?}");
            None
        }
    };

    std::thread::spawn(|| {
        let stdin = std::io::stdin();
        for line in stdin.lines() {
            let Ok(line) = line else { break };
            match line.trim() {
                "pause" => set_paused(true),
                "resume" => set_paused(false),
                "mute" => set_muted(true),
                "unmute" => set_muted(false),
                "debug" => set_debug_overlay(true),
                "nodebug" => set_debug_overlay(false),
                other if !other.is_empty() => println!("unknown command: {other}"),
                _ => {}
            }
        }
    });

    let mut seen: HashSet<Selection> = HashSet::new();
    let mut active: HashMap<String, Selection> = HashMap::new();

    let pending: Arc<Mutex<Option<Selection>>> = Arc::new(Mutex::new(None));
    {
        let pending = pending.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(500));
            if browse_ui_open() {
                continue;
            }
            let Some(sel) = pending.lock().unwrap().take() else {
                continue;
            };
            println!("browse UI closed, applying deferred selection: {sel:?}");
            swap_renderer(&sel.file, &sel.location, monitor);
        });
    }

    let mut buf = [0u8; 8192];
    loop {
        let n = match selection_socket.recv(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                println!("selection socket recv error: {e}");
                continue;
            }
        };

        let Some(sel) = extract_json(&String::from_utf8_lossy(&buf[..n])) else {
            continue;
        };

        if active.get(&sel.location) == Some(&sel) {
            continue;
        }
        active.insert(sel.location.clone(), sel.clone());

        if seen.contains(&sel) {
            println!("== repeat selection, still applying: {sel:?}");
        } else {
            println!("== new selection: {sel:?}");
        }
        seen.insert(sel.clone());
        save_selection(&sel);

        if browse_ui_open() {
            println!("browse UI still open, deferring apply for: {sel:?}");
            *pending.lock().unwrap() = Some(sel);
            continue;
        }

        swap_renderer(&sel.file, &sel.location, monitor);
    }
}
