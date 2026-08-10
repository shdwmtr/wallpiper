use std::io::Write;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixDatagram;

const SOCKET_PATH: &str = "/tmp/wallpiper-capture.sock";
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

macro_rules! logln {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        let _ = std::io::stdout().flush();
    }};
}

fn make_memfd(size: usize) -> RawFd {
    let name = c"wallpiper-shm-test";
    let fd = unsafe { libc::memfd_create(name.as_ptr(), 0) };
    assert!(fd >= 0, "memfd_create failed: {:?}", std::io::Error::last_os_error());
    let res = unsafe { libc::ftruncate(fd, size as libc::off_t) };
    assert_eq!(res, 0, "ftruncate failed: {:?}", std::io::Error::last_os_error());
    fd
}

fn fill_pattern(fd: RawFd, width: u32, height: u32, stride: u32, phase: u32) {
    let size = stride as usize * height as usize;
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };
    assert_ne!(ptr, libc::MAP_FAILED, "mmap failed");
    let buf = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, size) };
    for y in 0..height {
        for x in 0..width {
            let offset = (y * stride + x * 4) as usize;
            // Format_RGB32 byte order in memory: B, G, R, X (little-endian 0xffRRGGBB).
            let checker = (((x + phase) / 40) + (y / 40)) % 2 == 0;
            let (r, g, b) = if checker { (255u8, 32u8, 32u8) } else { (32u8, 255u8, 32u8) };
            buf[offset] = b;
            buf[offset + 1] = g;
            buf[offset + 2] = r;
            buf[offset + 3] = 0xff;
        }
    }
    unsafe { libc::munmap(ptr, size) };
}

fn send_with_fd(socket: &UnixDatagram, header: &str, fd: RawFd) {
    let header_bytes = header.as_bytes();
    let iov = libc::iovec {
        iov_base: header_bytes.as_ptr() as *mut libc::c_void,
        iov_len: header_bytes.len(),
    };
    let mut cmsg_buf = [0u8; 64];
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    let mut iov = iov;
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
    msg.msg_controllen = unsafe { libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) as usize };

    unsafe {
        let cmsg = libc::CMSG_FIRSTHDR(&msg);
        (*cmsg).cmsg_level = libc::SOL_SOCKET;
        (*cmsg).cmsg_type = libc::SCM_RIGHTS;
        (*cmsg).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<libc::c_int>() as u32) as usize;
        let data_ptr = libc::CMSG_DATA(cmsg) as *mut libc::c_int;
        std::ptr::write_unaligned(data_ptr, fd);
    }

    let n = unsafe { libc::sendmsg(socket.as_raw_fd(), &msg, 0) };
    logln!("sendmsg({header:?}) -> {n}");
    if n < 0 {
        logln!("  errno: {:?}", std::io::Error::last_os_error());
    }
}

fn main() {
    let stride = WIDTH * 4;
    logln!("connecting to {SOCKET_PATH}");
    let socket = UnixDatagram::unbound().expect("create dgram socket");
    socket.connect(SOCKET_PATH).unwrap_or_else(|e| {
        panic!("connect to {SOCKET_PATH} failed: {e} (is plasmashell running with the KDE wallpaper active?)")
    });

    let duration_secs: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(duration_secs);

    logln!("streaming a scrolling {WIDTH}x{HEIGHT} checkerboard for {duration_secs}s (~30fps) — drag a window over the wallpaper now");
    let mut phase: u32 = 0;
    let mut frame_count = 0u64;
    while std::time::Instant::now() < deadline {
        let fd = make_memfd(stride as usize * HEIGHT as usize);
        fill_pattern(fd, WIDTH, HEIGHT, stride, phase);
        let header = format!("SHM {WIDTH} {HEIGHT} {stride}\n");
        send_with_fd(&socket, &header, fd);
        unsafe { libc::close(fd) };
        phase = (phase + 4) % (WIDTH * 2);
        frame_count += 1;
        if frame_count % 30 == 0 {
            logln!("sent {frame_count} frames");
        }
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
    logln!("done, sent {frame_count} frames total");
}
