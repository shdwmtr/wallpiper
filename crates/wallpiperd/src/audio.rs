use std::process::Command;

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

pub fn set_muted(muted: bool) {
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
