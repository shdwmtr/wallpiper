use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::io::RawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc;

pub mod debug_overlay;
use std::time::Duration;

pub const CAPTURE_SOCKET_PATH: &str = "/tmp/wallpiper-capture.sock";

pub fn ctl_socket_path(portal_name: &str) -> String {
    format!("/tmp/wallpiper-portal-{portal_name}-ctl.sock")
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MonitorGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub logical_width: u32,
    pub logical_height: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum SocketEvent {
    Buf {
        slot: u32,
        width: u32,
        height: u32,
        stride: u32,
        modifier: u64,
        fd: RawFd,
        sync_fd: Option<RawFd>,
    },
    Frame {
        slot: u32,
        sync_fd: Option<RawFd>,
    },
    Shm {
        width: u32,
        height: u32,
        stride: u32,
        fd: RawFd,
    },
}

const MAX_RECV_FDS: usize = 2;

unsafe fn recv_msg(sock_fd: RawFd) -> std::io::Result<(String, Vec<RawFd>)> {
    let mut header_buf = [0u8; 256];
    let mut cmsg_buf = [0u8; 64];
    let mut iov = libc::iovec {
        iov_base: header_buf.as_mut_ptr() as *mut libc::c_void,
        iov_len: header_buf.len(),
    };
    let mut msg: libc::msghdr = std::mem::zeroed();
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
    msg.msg_controllen = cmsg_buf.len();

    let n = libc::recvmsg(sock_fd, &mut msg, 0);
    if n < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let header = String::from_utf8_lossy(&header_buf[..n as usize])
        .trim()
        .to_string();
    let cmsg = libc::CMSG_FIRSTHDR(&msg);
    let mut fds = Vec::new();
    if !cmsg.is_null() {
        let data_len = (*cmsg).cmsg_len.saturating_sub(libc::CMSG_LEN(0) as usize);
        let count = (data_len / std::mem::size_of::<libc::c_int>()).min(MAX_RECV_FDS);
        let data_ptr = libc::CMSG_DATA(cmsg) as *const libc::c_int;
        for i in 0..count {
            fds.push(std::ptr::read_unaligned(data_ptr.add(i)));
        }
    }
    Ok((header, fds))
}

pub fn bind_unix_dgram(path: &str) -> RawFd {
    let _ = std::fs::remove_file(path);
    let sock_fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, 0) };
    assert!(sock_fd >= 0, "failed to create unix dgram socket");

    let mut addr: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
    let path_bytes = path.as_bytes();
    assert!(
        path_bytes.len() < addr.sun_path.len(),
        "socket path too long: {path}"
    );
    for (i, &b) in path_bytes.iter().enumerate() {
        addr.sun_path[i] = b as libc::c_char;
    }
    let addr_len =
        (std::mem::size_of::<libc::sa_family_t>() + path_bytes.len() + 1) as libc::socklen_t;
    let bind_res = unsafe {
        libc::bind(
            sock_fd,
            &addr as *const _ as *const libc::sockaddr,
            addr_len,
        )
    };
    assert_eq!(
        bind_res,
        0,
        "failed to bind {path}: {:?}",
        std::io::Error::last_os_error()
    );
    sock_fd
}

pub fn parse_event(header: &str, fds: &[RawFd]) -> Option<SocketEvent> {
    let parts: Vec<&str> = header.split_whitespace().collect();
    match parts.first().copied() {
        Some("BUF") => Some(SocketEvent::Buf {
            slot: parts.get(1)?.parse().ok()?,
            width: parts.get(2)?.parse().ok()?,
            height: parts.get(3)?.parse().ok()?,
            stride: parts.get(5)?.parse().ok()?,
            modifier: parts.get(6)?.parse().ok()?,
            fd: *fds.first()?,
            sync_fd: fds.get(1).copied(),
        }),
        Some("FRAME") => Some(SocketEvent::Frame {
            slot: parts.get(1)?.parse().ok()?,
            sync_fd: fds.first().copied(),
        }),
        Some("SHM") => Some(SocketEvent::Shm {
            width: parts.get(1)?.parse().ok()?,
            height: parts.get(2)?.parse().ok()?,
            stride: parts.get(3)?.parse().ok()?,
            fd: *fds.first()?,
        }),
        _ => None,
    }
}

pub fn spawn_capture_socket_thread() -> mpsc::Receiver<SocketEvent> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let sock_fd = bind_unix_dgram(CAPTURE_SOCKET_PATH);
        println!("[socket] listening on {CAPTURE_SOCKET_PATH} (dgram)");

        loop {
            match unsafe { recv_msg(sock_fd) } {
                Ok((header, fds)) => match parse_event(&header, &fds) {
                    Some(event) => {
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                    None => {
                        println!("[socket] unrecognized or malformed message: {header:?}");
                        for fd in fds {
                            unsafe { libc::close(fd) };
                        }
                    }
                },
                Err(e) => println!("[socket] recvmsg failed: {e}"),
            }
        }
    });
    rx
}

#[derive(Debug, Clone)]
pub enum CtlRequest {
    Geometry,
    Detach,
    SetDebug(bool),
    CursorPos,
}

impl CtlRequest {
    pub fn parse(line: &str) -> Option<Self> {
        match line.trim() {
            "GEOMETRY" => Some(CtlRequest::Geometry),
            "DETACH" => Some(CtlRequest::Detach),
            "DEBUG_ON" => Some(CtlRequest::SetDebug(true)),
            "DEBUG_OFF" => Some(CtlRequest::SetDebug(false)),
            "CURSOR_POS" => Some(CtlRequest::CursorPos),
            _ => None,
        }
    }

    pub fn encode(&self) -> String {
        match self {
            CtlRequest::Geometry => "GEOMETRY\n".to_string(),
            CtlRequest::Detach => "DETACH\n".to_string(),
            CtlRequest::SetDebug(true) => "DEBUG_ON\n".to_string(),
            CtlRequest::SetDebug(false) => "DEBUG_OFF\n".to_string(),
            CtlRequest::CursorPos => "CURSOR_POS\n".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CtlResponse {
    Ok,
    Err(String),
    Geometry(MonitorGeometry),
    CursorPos { x: i32, y: i32 },
}

impl CtlResponse {
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line == "OK" {
            return Some(CtlResponse::Ok);
        }
        if let Some(rest) = line.strip_prefix("ERR") {
            return Some(CtlResponse::Err(rest.trim().to_string()));
        }
        if let Some(rest) = line.strip_prefix("GEOMETRY ") {
            return serde_json::from_str(rest).ok().map(CtlResponse::Geometry);
        }
        if let Some(rest) = line.strip_prefix("CURSOR_POS ") {
            let mut parts = rest.split_whitespace();
            let x: i32 = parts.next()?.parse().ok()?;
            let y: i32 = parts.next()?.parse().ok()?;
            return Some(CtlResponse::CursorPos { x, y });
        }
        None
    }

    pub fn encode(&self) -> String {
        match self {
            CtlResponse::Ok => "OK\n".to_string(),
            CtlResponse::Err(msg) => format!("ERR {msg}\n"),
            CtlResponse::Geometry(geometry) => {
                let json = serde_json::to_string(geometry).unwrap_or_default();
                format!("GEOMETRY {json}\n")
            }
            CtlResponse::CursorPos { x, y } => format!("CURSOR_POS {x} {y}\n"),
        }
    }
}

pub fn send_ctl_request(portal_name: &str, request: CtlRequest) -> Option<CtlResponse> {
    let stream = UnixStream::connect(ctl_socket_path(portal_name)).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .ok()?;
    let mut writer = stream.try_clone().ok()?;
    writer.write_all(request.encode().as_bytes()).ok()?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    CtlResponse::parse(&line)
}

pub fn spawn_ctl_listener(
    portal_name: &str,
    cursor_pos: impl Fn() -> Option<(i32, i32)> + Send + 'static,
) -> mpsc::Receiver<(CtlRequest, mpsc::Sender<CtlResponse>)> {
    let (tx, rx) = mpsc::channel();
    let path = ctl_socket_path(portal_name);
    std::thread::spawn(move || {
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

            let Some(request) = CtlRequest::parse(&line) else {
                let _ = stream.write_all(
                    CtlResponse::Err("unrecognized command".to_string())
                        .encode()
                        .as_bytes(),
                );
                continue;
            };

            if matches!(request, CtlRequest::CursorPos) {
                let response = match cursor_pos() {
                    Some((x, y)) => CtlResponse::CursorPos { x, y },
                    None => CtlResponse::Err("cursor position unavailable".to_string()),
                };
                let _ = stream.write_all(response.encode().as_bytes());
                continue;
            }

            let (reply_tx, reply_rx) = mpsc::channel();
            if tx.send((request, reply_tx)).is_err() {
                let _ = stream.write_all(
                    CtlResponse::Err("listener shutting down".to_string())
                        .encode()
                        .as_bytes(),
                );
                continue;
            }
            let response =
                reply_rx
                    .recv_timeout(Duration::from_secs(2))
                    .unwrap_or(CtlResponse::Err(
                        "timed out waiting for response".to_string(),
                    ));
            let _ = stream.write_all(response.encode().as_bytes());
        }
    });
    rx
}
