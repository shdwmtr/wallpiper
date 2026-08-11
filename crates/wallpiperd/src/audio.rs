use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::process::find_renderer_pid;

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

fn apply_mute(sink_input_id: &str, muted: bool) {
    let val = if muted { "1" } else { "0" };
    let status = Command::new("pactl")
        .args(["set-sink-input-mute", sink_input_id, val])
        .status();
    println!("set-sink-input-mute {sink_input_id} {val} -> {status:?}");
}

fn apply_volume(sink_input_id: &str, volume: u8) {
    let pct = format!("{volume}%");
    let status = Command::new("pactl")
        .args(["set-sink-input-volume", sink_input_id, &pct])
        .status();
    println!("set-sink-input-volume {sink_input_id} {pct} -> {status:?}");
}

pub fn set_muted(muted: bool) {
    let Some(pid) = find_renderer_pid() else {
        println!("mute: no active renderer found");
        return;
    };
    let Some(sink_input_id) = find_renderer_sink_input(pid) else {
        println!("mute: no audio stream found for renderer pid={pid}");
        return;
    };
    apply_mute(&sink_input_id, muted);
}

pub fn set_volume(volume: u8) {
    let Some(pid) = find_renderer_pid() else {
        println!("volume: no active renderer found");
        return;
    };
    let Some(sink_input_id) = find_renderer_sink_input(pid) else {
        println!("volume: no audio stream found for renderer pid={pid}");
        return;
    };
    apply_volume(&sink_input_id, volume);
}

const PID_WAIT_ATTEMPTS: u32 = 120;
const PID_WAIT_INTERVAL: Duration = Duration::from_secs(1);
const SINK_INPUT_WAIT_TIMEOUT: Duration = Duration::from_secs(60);

fn wait_for_renderer_pid() -> Option<i32> {
    for _ in 0..PID_WAIT_ATTEMPTS {
        if let Some(pid) = find_renderer_pid() {
            return Some(pid);
        }
        std::thread::sleep(PID_WAIT_INTERVAL);
    }
    None
}

/// Blocks until the renderer's sink input appears (or `SINK_INPUT_WAIT_TIMEOUT`
/// elapses), applying `volume`/`muted` the moment it's created. Uses `pactl
/// subscribe` rather than polling so the state lands before the stream is
/// connected to a sink and can produce any audible output, instead of racing
/// against a fixed poll interval.
fn wait_for_sink_input(pid: i32, volume: u8, muted: bool) -> bool {
    if let Some(sink_input_id) = find_renderer_sink_input(pid) {
        apply_volume(&sink_input_id, volume);
        apply_mute(&sink_input_id, muted);
        return true;
    }

    let Ok(mut child) = Command::new("pactl")
        .arg("subscribe")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        return false;
    };

    let child_pid = child.id() as i32;
    let done = Arc::new(AtomicBool::new(false));
    let watchdog_done = done.clone();
    std::thread::spawn(move || {
        std::thread::sleep(SINK_INPUT_WAIT_TIMEOUT);
        if !watchdog_done.load(Ordering::SeqCst) {
            unsafe { libc::kill(child_pid, libc::SIGKILL) };
        }
    });

    let mut found = false;
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        if !line.contains("sink-input") {
            continue;
        }
        if let Some(sink_input_id) = find_renderer_sink_input(pid) {
            apply_volume(&sink_input_id, volume);
            apply_mute(&sink_input_id, muted);
            found = true;
            break;
        }
    }

    done.store(true, Ordering::SeqCst);
    let _ = child.kill();
    let _ = child.wait();
    found
}

pub fn apply_saved_state(volume: u8, muted: bool) {
    std::thread::spawn(move || {
        let Some(pid) = wait_for_renderer_pid() else {
            let waited = PID_WAIT_ATTEMPTS * PID_WAIT_INTERVAL.as_secs() as u32;
            println!(
                "audio: renderer never started after {waited}s, giving up on restoring saved volume/mute"
            );
            return;
        };

        if !wait_for_sink_input(pid, volume, muted) {
            println!(
                "audio: renderer pid={pid} never registered an audio stream after {}s, giving up on restoring saved volume/mute",
                SINK_INPUT_WAIT_TIMEOUT.as_secs()
            );
        }
    });
}
