use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Duration;

use crate::runtime_dir;

pub fn daemon_ctl_socket_path() -> String {
    format!("{}/wallpiperd-ctl.sock", runtime_dir())
}

#[derive(Debug, Clone)]
pub enum DaemonResponse {
    Ok,
    Err(String),
}

impl DaemonResponse {
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line == "OK" {
            return Some(DaemonResponse::Ok);
        }
        line.strip_prefix("ERR")
            .map(|rest| DaemonResponse::Err(rest.trim().to_string()))
    }

    pub fn encode(&self) -> String {
        match self {
            DaemonResponse::Ok => "OK\n".to_string(),
            DaemonResponse::Err(msg) => format!("ERR {msg}\n"),
        }
    }
}

pub fn send_daemon_command(args: &[&str]) -> Result<(), String> {
    let path = daemon_ctl_socket_path();
    let stream = UnixStream::connect(&path)
        .map_err(|e| format!("wallpiperd is not running (no socket at {path}: {e})"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;

    let mut writer = stream.try_clone().map_err(|e| e.to_string())?;
    writer
        .write_all(format!("{}\n", args.join(" ")).as_bytes())
        .map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;

    match DaemonResponse::parse(&line) {
        Some(DaemonResponse::Ok) => Ok(()),
        Some(DaemonResponse::Err(msg)) => Err(msg),
        None => Err(format!("malformed response from wallpiperd: {line:?}")),
    }
}

pub fn spawn_daemon_ctl_listener(handler: impl Fn(&[&str]) -> Result<(), String> + Send + 'static) {
    let path = daemon_ctl_socket_path();
    std::thread::spawn(move || {
        if let Some(dir) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::remove_file(&path);
        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                println!("[ctl] failed to bind {path}: {e}");
                return;
            }
        };
        println!("[ctl] listening on {path}");

        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let Ok(reader_stream) = stream.try_clone() else {
                continue;
            };
            let mut reader = BufReader::new(reader_stream);
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(n) if n > 0 => {}
                _ => continue,
            }

            let args: Vec<&str> = line.split_whitespace().collect();
            let response = if args.is_empty() {
                DaemonResponse::Err("empty command".to_string())
            } else {
                match handler(&args) {
                    Ok(()) => DaemonResponse::Ok,
                    Err(msg) => DaemonResponse::Err(msg),
                }
            };
            let _ = stream.write_all(response.encode().as_bytes());
        }
    });
}
