use std::time::Duration;

fn pid_alive(pid: i32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

pub fn kill_pids_gracefully(pids: &[i32]) {
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

pub fn find_renderer_pids() -> Vec<i32> {
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

pub fn find_renderer_pid() -> Option<i32> {
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

pub fn find_renderer_pids_for_tag(tag: &str) -> Vec<i32> {
    find_pids_for_tag(tag)
        .into_iter()
        .filter(|&pid| {
            std::fs::read_to_string(format!("/proc/{pid}/comm"))
                .map(|comm| comm.trim() == "wallpaper64.exe")
                .unwrap_or(false)
        })
        .collect()
}

pub fn find_webwallpaper_pids() -> Vec<i32> {
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
