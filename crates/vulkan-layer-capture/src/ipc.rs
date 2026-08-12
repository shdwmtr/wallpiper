use std::io::{self, IoSlice};
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixDatagram;
use std::sync::Mutex;

use ash::vk;
use nix::sys::socket::{sendmsg, ControlMessage, MsgFlags, UnixAddr};

use crate::config::CAPTURE_SOCKET_PATH;
use crate::logging::log;

pub(crate) struct BufferAnnounce {
    pub slot: u32,
    pub width: u32,
    pub height: u32,
    pub format: vk::Format,
    pub stride: u32,
    pub modifier: u64,
    pub fd: RawFd,
    pub sync_fd: RawFd,
}

#[derive(Default)]
pub(crate) struct CaptureLink {
    socket: Mutex<Option<UnixDatagram>>,
}

impl CaptureLink {
    fn with_socket<T>(&self, f: impl FnOnce(&UnixDatagram) -> io::Result<T>) -> io::Result<T> {
        let mut guard = self.socket.lock().unwrap();
        if guard.is_none() {
            let socket = UnixDatagram::unbound()?;
            socket.connect(CAPTURE_SOCKET_PATH)?;
            log!("connected to capture socket");
            *guard = Some(socket);
        }
        let result = f(guard.as_ref().unwrap());
        if result.is_err() {
            *guard = None;
        }
        result
    }

    pub(crate) fn notify_frame(&self, slot: u32, sync_fd: RawFd) {
        let header = format!("FRAME {slot}\n");
        let result = if sync_fd >= 0 {
            self.with_socket(|socket| {
                send_with_fds(socket.as_raw_fd(), header.as_bytes(), &[sync_fd])
            })
        } else {
            self.with_socket(|socket| socket.send(header.as_bytes()).map(|_| ()))
        };
        if sync_fd >= 0 {
            unsafe { libc::close(sync_fd) };
        }
        if let Err(e) = result {
            log!("notify_frame failed: {e}");
        }
    }

    pub(crate) fn notify_buffer(&self, announce: BufferAnnounce) -> bool {
        let BufferAnnounce {
            slot,
            width,
            height,
            format,
            stride,
            modifier,
            fd,
            sync_fd,
        } = announce;
        let header = format!(
            "BUF {slot} {width} {height} {} {stride} {modifier}\n",
            format.as_raw()
        );
        let fds: &[RawFd] = if sync_fd >= 0 { &[fd, sync_fd] } else { &[fd] };
        let result =
            self.with_socket(|socket| send_with_fds(socket.as_raw_fd(), header.as_bytes(), fds));
        unsafe { libc::close(fd) };
        if sync_fd >= 0 {
            unsafe { libc::close(sync_fd) };
        }
        match result {
            Ok(()) => {
                log!("sent BUF for slot {slot}");
                true
            }
            Err(e) => {
                log!("notify_buffer failed: {e}");
                false
            }
        }
    }
}

fn send_with_fds(socket: RawFd, header: &[u8], fds: &[RawFd]) -> io::Result<()> {
    let iov = [IoSlice::new(header)];
    let cmsgs = [ControlMessage::ScmRights(fds)];
    sendmsg::<UnixAddr>(socket, &iov, &cmsgs, MsgFlags::empty(), None)
        .map(|_| ())
        .map_err(io::Error::from)
}
