use std::collections::HashMap;
use std::os::unix::io::{FromRawFd, OwnedFd, RawFd};
use std::process::Command;
use std::time::Duration;

use serde_json::Value;
use x11rb::connection::Connection;
use x11rb::protocol::dri3::ConnectionExt as _;
use x11rb::protocol::shm::ConnectionExt as _;
use x11rb::protocol::xproto::{
    self, BackingStore, ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt as _,
    CreateGCAux, CreateWindowAux, EventMask, ImageFormat, StackMode, WindowClass,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use wallpiper_protocol::debug_overlay;
use wallpiper_protocol::{CtlRequest, CtlResponse, MonitorGeometry, SocketEvent};

struct SlotPixmap {
    pixmap: xproto::Pixmap,
    width: u32,
    height: u32,
}

struct ShmPixmap {
    pixmap: xproto::Pixmap,
    seg: x11rb::protocol::shm::Seg,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy)]
enum DisplaySource {
    Slot(u32),
    Shm,
}

struct BufMsg {
    slot: u32,
    width: u32,
    height: u32,
    stride: u32,
    modifier: u64,
    fd: RawFd,
}

#[allow(clippy::too_many_arguments)]
fn import_dri3_pixmap(
    conn: &RustConnection,
    pixmap: xproto::Pixmap,
    window: xproto::Window,
    width: u32,
    height: u32,
    stride: u32,
    depth: u8,
    modifier: u64,
    fd: RawFd,
) -> Result<(), String> {
    let owned_fd = unsafe { OwnedFd::from_raw_fd(fd) };
    let cookie = conn
        .dri3_pixmap_from_buffers(
            pixmap,
            window,
            width as u16,
            height as u16,
            stride,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            depth,
            32,
            modifier,
            vec![owned_fd],
        )
        .map_err(|e| e.to_string())?;
    cookie.check().map_err(|e| e.to_string())
}

fn import_shm_pixmap(
    conn: &RustConnection,
    pixmap: xproto::Pixmap,
    window: xproto::Window,
    width: u32,
    height: u32,
    depth: u8,
    seg: x11rb::protocol::shm::Seg,
) -> Result<(), String> {
    let cookie = conn
        .shm_create_pixmap(pixmap, window, width as u16, height as u16, depth, seg, 0)
        .map_err(|e| e.to_string())?;
    cookie.check().map_err(|e| e.to_string())
}

fn detect_geometry() -> MonitorGeometry {
    for attempt in 1..=3 {
        if let Some(geometry) = try_detect_geometry() {
            return geometry;
        }
        println!("monitor detection attempt {attempt} failed, retrying");
        std::thread::sleep(Duration::from_millis(500));
    }
    println!("monitor detection failed after retries, falling back to 1920x1080 at 0,0");
    MonitorGeometry {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        logical_width: 1920,
        logical_height: 1080,
    }
}

fn try_detect_geometry() -> Option<MonitorGeometry> {
    let workspaces_out = Command::new("i3-msg")
        .args(["-t", "get_workspaces"])
        .output()
        .ok()?;
    let workspaces: Value = serde_json::from_slice(&workspaces_out.stdout).ok()?;
    let focused = workspaces
        .as_array()?
        .iter()
        .find(|w| w["focused"] == true)?;
    let output_name = focused["output"].as_str()?;

    let outputs_out = Command::new("i3-msg")
        .args(["-t", "get_outputs"])
        .output()
        .ok()?;
    let outputs: Value = serde_json::from_slice(&outputs_out.stdout).ok()?;
    let output = outputs
        .as_array()?
        .iter()
        .find(|o| o["name"].as_str() == Some(output_name) && o["active"] == true)?;

    let rect = &output["rect"];
    let x = rect["x"].as_i64()? as i32;
    let y = rect["y"].as_i64()? as i32;
    let width = rect["width"].as_u64()? as u32;
    let height = rect["height"].as_u64()? as u32;
    Some(MonitorGeometry {
        x,
        y,
        width,
        height,
        logical_width: width,
        logical_height: height,
    })
}

fn query_cursor_pos() -> Option<(i32, i32)> {
    let (conn, screen_num) = RustConnection::connect(None).ok()?;
    let root = conn.setup().roots.get(screen_num)?.root;
    let reply = conn.query_pointer(root).ok()?.reply().ok()?;
    Some((reply.root_x as i32, reply.root_y as i32))
}

fn main() {
    let geometry = detect_geometry();
    println!("detected monitor geometry: {geometry:?}");

    let (conn, screen_num) = RustConnection::connect(None).expect("connect to X11");
    let (root, depth, visual) = {
        let screen = &conn.setup().roots[screen_num];
        (screen.root, screen.root_depth, screen.root_visual)
    };

    conn.dri3_query_version(1, 2)
        .expect("send dri3 query_version")
        .reply()
        .expect("DRI3 extension not available");
    conn.shm_query_version()
        .expect("send shm query_version")
        .reply()
        .expect("MIT-SHM extension not available");

    let window = conn.generate_id().expect("generate window id");
    let win_aux = CreateWindowAux::new()
        .background_pixel(0)
        .override_redirect(1)
        .event_mask(EventMask::EXPOSURE)
        .backing_store(BackingStore::ALWAYS);
    conn.create_window(
        depth,
        window,
        root,
        geometry.x as i16,
        geometry.y as i16,
        geometry.width as u16,
        geometry.height as u16,
        0,
        WindowClass::INPUT_OUTPUT,
        visual,
        &win_aux,
    )
    .expect("send create_window")
    .check()
    .expect("create_window failed");

    conn.map_window(window).expect("map_window");
    conn.configure_window(
        window,
        &ConfigureWindowAux::new().stack_mode(StackMode::BELOW),
    )
    .expect("configure_window");

    let gc = conn.generate_id().expect("generate gc id");
    conn.create_gc(gc, window, &CreateGCAux::default())
        .expect("create_gc");
    conn.flush().expect("flush");

    println!(
        "background window {window} mapped at {}x{}+{}+{}",
        geometry.width, geometry.height, geometry.x, geometry.y
    );

    let event_rx = wallpiper_protocol::spawn_capture_socket_thread();
    let ctl_rx = wallpiper_protocol::spawn_ctl_listener("i3", query_cursor_pos);

    let mut state = State {
        conn,
        window,
        gc,
        depth,
        geometry,
        slots: HashMap::new(),
        current_shm_pixmap: None,
        current_source: None,
        debug_enabled: false,
        debug_throttle: debug_overlay::DebugThrottle::new(),
        stats: debug_overlay::FrameStats::new(),
    };

    loop {
        while let Ok(Some(event)) = state.conn.poll_for_event() {
            state.handle_x_event(event);
        }

        match event_rx.recv_timeout(Duration::from_millis(16)) {
            Ok(event) => state.handle_event(event),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
        }
        while let Ok(event) = event_rx.try_recv() {
            state.handle_event(event);
        }

        if let Ok((request, reply_tx)) = ctl_rx.try_recv() {
            let response = match request {
                CtlRequest::Geometry => CtlResponse::Geometry(state.geometry),
                CtlRequest::Detach => {
                    state.detach();
                    CtlResponse::Ok
                }
                CtlRequest::SetDebug(enabled) => {
                    state.set_debug_enabled(enabled);
                    CtlResponse::Ok
                }
                CtlRequest::CursorPos => CtlResponse::Err("handled by ctl listener".to_string()),
            };
            let _ = reply_tx.send(response);
        }

        if state.debug_enabled {
            state.maybe_redraw_debug();
        }

        let _ = state.conn.flush();
    }
}

struct State {
    conn: RustConnection,
    window: xproto::Window,
    gc: xproto::Gcontext,
    depth: u8,
    geometry: MonitorGeometry,
    slots: HashMap<u32, SlotPixmap>,
    current_shm_pixmap: Option<ShmPixmap>,
    current_source: Option<DisplaySource>,
    debug_enabled: bool,
    debug_throttle: debug_overlay::DebugThrottle,
    stats: debug_overlay::FrameStats,
}

impl State {
    fn handle_x_event(&mut self, event: Event) {
        match event {
            Event::Expose(e) if e.window == self.window && e.count == 0 => {
                self.refresh_buffer();
            }
            Event::DestroyNotify(e) if e.window == self.window => {
                println!("background window destroyed, exiting");
                std::process::exit(0);
            }
            Event::Error(e) => {
                println!("[x11] protocol error: {e:?}");
            }
            _ => {}
        }
    }

    fn handle_event(&mut self, event: SocketEvent) {
        self.stats.record_capture();

        match event {
            SocketEvent::Buf {
                slot,
                width,
                height,
                stride,
                modifier,
                fd,
                sync_fd,
            } => {
                if let Some(sync_fd) = sync_fd {
                    unsafe { libc::close(sync_fd) };
                }
                self.handle_buf(BufMsg {
                    slot,
                    width,
                    height,
                    stride,
                    modifier,
                    fd,
                });
            }
            SocketEvent::Frame { slot, sync_fd } => {
                if let Some(sync_fd) = sync_fd {
                    unsafe { libc::close(sync_fd) };
                }
                if self.slots.contains_key(&slot) {
                    self.set_current_source(DisplaySource::Slot(slot));
                }
            }
            SocketEvent::Shm {
                width,
                height,
                stride,
                fd,
            } => {
                self.handle_shm(width, height, stride, fd);
            }
        }
    }

    fn handle_buf(&mut self, msg: BufMsg) {
        let BufMsg {
            slot,
            width,
            height,
            stride,
            modifier,
            fd,
        } = msg;

        if let Some(old) = self.slots.remove(&slot) {
            let _ = self.conn.free_pixmap(old.pixmap);
        }

        let pixmap = self.conn.generate_id().expect("generate pixmap id");
        let checked = import_dri3_pixmap(
            &self.conn,
            pixmap,
            self.window,
            width,
            height,
            stride,
            self.depth,
            modifier,
            fd,
        );
        match checked {
            Ok(()) => {
                println!(
                    "[socket] registered capture slot {slot} {width}x{height} stride={stride} modifier={modifier}"
                );
                self.slots.insert(
                    slot,
                    SlotPixmap {
                        pixmap,
                        width,
                        height,
                    },
                );
                self.set_current_source(DisplaySource::Slot(slot));
            }
            Err(e) => {
                println!("[socket] dri3 pixmap import failed for slot {slot}: {e}");
            }
        }
    }

    fn handle_shm(&mut self, width: u32, height: u32, stride: u32, fd: RawFd) {
        if stride != width * 4 {
            println!(
                "[socket] shm frame stride {stride} doesn't match tightly-packed {width}x4, \
                 X11's MIT-SHM CreatePixmap can't represent padded rows - dropping frame"
            );
            unsafe { libc::close(fd) };
            return;
        }

        let seg = self.conn.generate_id().expect("generate shm seg id");
        let owned_fd = unsafe { OwnedFd::from_raw_fd(fd) };
        if let Err(e) = self.conn.shm_attach_fd(seg, owned_fd, true) {
            println!("[socket] shm attach_fd failed: {e}");
            return;
        }

        let pixmap = self.conn.generate_id().expect("generate pixmap id");
        let checked = import_shm_pixmap(
            &self.conn,
            pixmap,
            self.window,
            width,
            height,
            self.depth,
            seg,
        );
        match checked {
            Ok(()) => {
                if let Some(old) = self.current_shm_pixmap.replace(ShmPixmap {
                    pixmap,
                    seg,
                    width,
                    height,
                }) {
                    let _ = self.conn.free_pixmap(old.pixmap);
                    let _ = self.conn.shm_detach(old.seg);
                }
                self.set_current_source(DisplaySource::Shm);
            }
            Err(e) => {
                println!("[socket] shm pixmap import failed: {e}");
                let _ = self.conn.shm_detach(seg);
            }
        }
    }

    fn set_current_source(&mut self, source: DisplaySource) {
        self.current_source = Some(source);
        self.refresh_buffer();
    }

    fn refresh_buffer(&mut self) {
        let Some(source) = self.current_source else {
            return;
        };
        let resolved = match source {
            DisplaySource::Slot(slot) => self
                .slots
                .get(&slot)
                .map(|sb| (sb.pixmap, sb.width, sb.height)),
            DisplaySource::Shm => self
                .current_shm_pixmap
                .as_ref()
                .map(|sb| (sb.pixmap, sb.width, sb.height)),
        };
        let Some((pixmap, width, height)) = resolved else {
            return;
        };

        let _ = self.conn.copy_area(
            pixmap,
            self.window,
            self.gc,
            0,
            0,
            0,
            0,
            width as u16,
            height as u16,
        );
        let _ = self.conn.flush();

        self.stats.record_display();

        if self.debug_enabled {
            self.draw_debug_overlay();
        }
    }

    fn detach(&mut self) {
        for (_, slot) in self.slots.drain() {
            let _ = self.conn.free_pixmap(slot.pixmap);
        }
        if let Some(shm) = self.current_shm_pixmap.take() {
            let _ = self.conn.free_pixmap(shm.pixmap);
            let _ = self.conn.shm_detach(shm.seg);
        }
        self.current_source = None;

        let _ = self.conn.change_window_attributes(
            self.window,
            &ChangeWindowAttributesAux::new().background_pixel(0),
        );
        let _ = self.conn.clear_area(false, self.window, 0, 0, 0, 0);
        let _ = self.conn.flush();
        println!("[ctl] detached, released all buffers");
    }

    fn set_debug_enabled(&mut self, enabled: bool) {
        self.debug_enabled = enabled;
        self.debug_throttle.reset();
        if enabled {
            self.draw_debug_overlay();
        } else {
            self.refresh_buffer();
        }
        println!("[ctl] debug overlay -> {enabled}");
    }

    fn maybe_redraw_debug(&mut self) {
        if self.debug_throttle.should_redraw() {
            self.draw_debug_overlay();
        }
    }

    fn draw_debug_overlay(&mut self) {
        let pixels = debug_overlay::render_stats_panel(&self.stats);
        let overlay_y = ((self.geometry.height as i32 - debug_overlay::HEIGHT as i32) / 2).max(0);
        let _ = self.conn.put_image(
            ImageFormat::Z_PIXMAP,
            self.window,
            self.gc,
            debug_overlay::WIDTH,
            debug_overlay::HEIGHT,
            12,
            overlay_y as i16,
            0,
            self.depth,
            &pixels,
        );
        let _ = self.conn.flush();
    }
}
