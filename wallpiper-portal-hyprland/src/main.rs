use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::os::unix::io::{BorrowedFd, FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, FrameCallbackData},
    delegate_registry,
    dmabuf::{DmabufFeedback, DmabufHandler, DmabufState},
    globals::GlobalData,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{
        wl_buffer, wl_output, wl_shm, wl_shm_pool, wl_subcompositor, wl_subsurface, wl_surface,
    },
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_buffer_params_v1;
use wayland_protocols::wp::viewporter::client::{
    wp_viewport::WpViewport, wp_viewporter::WpViewporter,
};

use wallpiper_protocol::{CtlRequest, CtlResponse, MonitorGeometry, SocketEvent};

const DRM_FORMAT_XRGB8888: u32 = 0x34325258;

const DEBUG_OVERLAY_WIDTH: i32 = 210;
const DEBUG_OVERLAY_HEIGHT: i32 = 120;
const DEBUG_REDRAW_INTERVAL: Duration = Duration::from_millis(250);
const DEBUG_STATS_WINDOW: Duration = Duration::from_secs(3);
const DEBUG_SPARKLINE_SAMPLES: usize = 40;

#[derive(Clone, Copy)]
struct FrameInfo {
    fd: RawFd,
    width: u32,
    height: u32,
    stride: u32,
}

#[derive(Clone, Copy)]
enum DisplaySource {
    Slot(u32),
    Shm(FrameInfo),
}

struct SlotBuffer {
    buffer: wl_buffer::WlBuffer,
    fd: RawFd,
    width: u32,
    height: u32,
}

struct BufMsg {
    slot: u32,
    width: u32,
    height: u32,
    stride: u32,
    modifier: u64,
    fd: RawFd,
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

fn geometry_from_scale(x: i32, y: i32, width: u32, height: u32, scale: f64) -> MonitorGeometry {
    let scale = if scale > 0.0 { scale } else { 1.0 };
    let logical_width = (width as f64 / scale).round() as u32;
    let logical_height = (height as f64 / scale).round() as u32;
    MonitorGeometry {
        x,
        y,
        width,
        height,
        logical_width,
        logical_height,
    }
}

fn hypr_socket_path() -> Option<String> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    Some(format!("{runtime_dir}/hypr/{sig}/.socket.sock"))
}

fn query_cursor_pos() -> Option<(i32, i32)> {
    let path = hypr_socket_path()?;
    let mut stream = UnixStream::connect(path).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .ok()?;
    stream
        .set_write_timeout(Some(Duration::from_millis(200)))
        .ok()?;
    stream.write_all(b"cursorpos").ok()?;
    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;
    let mut parts = response.trim().split(", ");
    let x: i32 = parts.next()?.parse().ok()?;
    let y: i32 = parts.next()?.parse().ok()?;
    Some((x, y))
}

fn try_detect_geometry() -> Option<MonitorGeometry> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()?;
    let monitors: Value = serde_json::from_slice(&output.stdout).ok()?;
    let focused = monitors.as_array()?.iter().find(|m| m["focused"] == true)?;
    let x = focused["x"].as_i64()? as i32;
    let y = focused["y"].as_i64()? as i32;
    let width = focused["width"].as_u64()? as u32;
    let height = focused["height"].as_u64()? as u32;
    let scale = focused["scale"].as_f64().unwrap_or(1.0);
    Some(geometry_from_scale(x, y, width, height, scale))
}

fn main() {
    let geometry = detect_geometry();
    println!("detected monitor geometry: {geometry:?}");

    let conn = Connection::connect_to_env().expect("connect to wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("registry_queue_init");
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("zwlr_layer_shell_v1 not available");
    let dmabuf_state = DmabufState::new(&globals, &qh);
    let viewporter = globals
        .bind::<WpViewporter, _, _>(&qh, 1..=1, GlobalData)
        .ok();
    if viewporter.is_none() {
        println!("wp_viewporter not available, buffer will be shown at native size (may overhang on fractional-scale outputs)");
    }
    let shm = globals
        .bind::<wl_shm::WlShm, _, _>(&qh, 1..=1, GlobalData)
        .ok();
    if shm.is_none() {
        println!("wl_shm not available, SHM-sourced frames (e.g. web wallpapers) won't display");
    }
    let subcompositor = globals
        .bind::<wl_subcompositor::WlSubcompositor, _, _>(&qh, 1..=1, GlobalData)
        .ok();
    if subcompositor.is_none() {
        println!("wl_subcompositor not available, debug overlay won't be available");
    }

    let mut state = State {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        compositor,
        layer_shell,
        dmabuf_state,
        shm,
        subcompositor,
        layer: None,
        viewport: None,
        geometry,
        width: 1920,
        height: 1080,
        slots: HashMap::new(),
        current_source: None,
        current_shm_buffer: None,
        current_shm_pool: None,
        frame_loop_running: false,
        debug_enabled: false,
        debug_subsurface: None,
        debug_surface: None,
        debug_shm_pool: None,
        debug_shm_buffer: None,
        debug_shm_fd: None,
        last_debug_draw: None,
        display_times: VecDeque::new(),
        capture_times: VecDeque::new(),
    };

    let surface = state.compositor.create_surface(&qh);
    if let Some(viewporter) = &viewporter {
        state.viewport = Some(viewporter.get_viewport(&surface, &qh, GlobalData));
    }
    let layer = state.layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Background,
        Some("wallpiper-portal-hyprland"),
        None,
    );
    layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
    layer.set_exclusive_zone(-1);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.set_size(0, 0);
    layer.commit();
    state.layer = Some(layer);

    println!("layer surface created, waiting for configure + frame");
    println!(
        "dmabuf protocol version: {:?}",
        state.dmabuf_state.version()
    );
    println!(
        "dmabuf modifiers (only populated if version <4): {:?}",
        state.dmabuf_state.modifiers()
    );

    let event_rx = wallpiper_protocol::spawn_capture_socket_thread();
    let ctl_rx = wallpiper_protocol::spawn_ctl_listener("hyprland", query_cursor_pos);

    loop {
        event_queue.dispatch_pending(&mut state).expect("dispatch");
        if let Err(e) = conn.flush() {
            let is_would_block = matches!(&e, wayland_client::backend::WaylandError::Io(io_err) if io_err.kind() == std::io::ErrorKind::WouldBlock);
            if !is_would_block {
                panic!("flush: {e}");
            }
        }

        match event_rx.recv_timeout(Duration::from_millis(16)) {
            Ok(event) => state.handle_event(&qh, event),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
        }

        if let Ok((request, reply_tx)) = ctl_rx.try_recv() {
            let response = match request {
                CtlRequest::Geometry => CtlResponse::Geometry(state.geometry),
                CtlRequest::Detach => {
                    state.detach();
                    CtlResponse::Ok
                }
                CtlRequest::SetDebug(enabled) => {
                    state.set_debug_enabled(&qh, enabled);
                    CtlResponse::Ok
                }
                CtlRequest::CursorPos => CtlResponse::Err("handled by ctl listener".to_string()),
            };
            let _ = reply_tx.send(response);
        }

        if state.debug_enabled {
            state.maybe_redraw_debug(&qh);
        }

        if let Some(guard) = event_queue.prepare_read() {
            let _ = guard.read();
        }
    }
}

struct State {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor: CompositorState,
    layer_shell: LayerShell,
    dmabuf_state: DmabufState,
    shm: Option<wl_shm::WlShm>,
    subcompositor: Option<wl_subcompositor::WlSubcompositor>,
    layer: Option<LayerSurface>,
    viewport: Option<WpViewport>,
    geometry: MonitorGeometry,
    width: u32,
    height: u32,
    slots: HashMap<u32, SlotBuffer>,
    current_source: Option<DisplaySource>,
    current_shm_buffer: Option<wl_buffer::WlBuffer>,
    current_shm_pool: Option<wl_shm_pool::WlShmPool>,
    frame_loop_running: bool,
    debug_enabled: bool,
    debug_subsurface: Option<wl_subsurface::WlSubsurface>,
    debug_surface: Option<wl_surface::WlSurface>,
    debug_shm_pool: Option<wl_shm_pool::WlShmPool>,
    debug_shm_buffer: Option<wl_buffer::WlBuffer>,
    debug_shm_fd: Option<RawFd>,
    last_debug_draw: Option<Instant>,
    display_times: VecDeque<Instant>,
    capture_times: VecDeque<Instant>,
}

impl State {
    fn set_current_source(&mut self, qh: &QueueHandle<Self>, source: DisplaySource) {
        self.current_source = Some(source);
        if !self.frame_loop_running {
            self.frame_loop_running = true;
            self.refresh_buffer(qh);
        }
    }

    fn handle_event(&mut self, qh: &QueueHandle<Self>, event: SocketEvent) {
        let now = Instant::now();
        self.capture_times.push_back(now);
        while let Some(&front) = self.capture_times.front() {
            if now.duration_since(front) > DEBUG_STATS_WINDOW {
                self.capture_times.pop_front();
            } else {
                break;
            }
        }

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
                self.handle_buf(
                    qh,
                    BufMsg {
                        slot,
                        width,
                        height,
                        stride,
                        modifier,
                        fd,
                    },
                );
            }
            SocketEvent::Frame { slot, sync_fd } => {
                if let Some(sync_fd) = sync_fd {
                    unsafe { libc::close(sync_fd) };
                }
                if self.slots.contains_key(&slot) {
                    self.set_current_source(qh, DisplaySource::Slot(slot));
                }
            }
            SocketEvent::Shm {
                width,
                height,
                stride,
                fd,
            } => {
                self.set_current_source(
                    qh,
                    DisplaySource::Shm(FrameInfo {
                        fd,
                        width,
                        height,
                        stride,
                    }),
                );
            }
        }
    }

    fn handle_buf(&mut self, qh: &QueueHandle<Self>, msg: BufMsg) {
        let BufMsg {
            slot,
            width,
            height,
            stride,
            modifier,
            fd,
        } = msg;
        if let Some(old) = self.slots.remove(&slot) {
            old.buffer.destroy();
            unsafe { libc::close(old.fd) };
        }

        let Ok(params) = self.dmabuf_state.create_params(qh) else {
            println!("zwp_linux_dmabuf_v1 not available");
            unsafe { libc::close(fd) };
            return;
        };
        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
        params.add(borrowed_fd, 0, 0, stride, modifier);
        let (buffer, _params) = params.create_immed(
            width as i32,
            height as i32,
            DRM_FORMAT_XRGB8888,
            zwp_linux_buffer_params_v1::Flags::empty(),
            qh,
        );

        println!("[socket] registered capture slot {slot} {width}x{height} stride={stride} modifier={modifier}");
        self.slots.insert(
            slot,
            SlotBuffer {
                buffer,
                fd,
                width,
                height,
            },
        );
        self.set_current_source(qh, DisplaySource::Slot(slot));
    }

    fn detach(&mut self) {
        if let Some(layer) = &self.layer {
            let surface = layer.wl_surface();
            surface.attach(None, 0, 0);
            surface.commit();
        }
        self.current_source = None;
        self.frame_loop_running = false;
        if let Some(buffer) = self.current_shm_buffer.take() {
            buffer.destroy();
        }
        if let Some(pool) = self.current_shm_pool.take() {
            pool.destroy();
        }
        for (_, slot) in self.slots.drain() {
            slot.buffer.destroy();
            unsafe { libc::close(slot.fd) };
        }
        println!("[ctl] detached from compositor, released all buffers");
    }

    fn refresh_buffer(&mut self, qh: &QueueHandle<Self>) {
        let Some(ref layer) = self.layer else {
            return;
        };
        let Some(source) = self.current_source else {
            return;
        };

        let (buffer, width, height, next_shm_buffer) = match source {
            DisplaySource::Slot(slot) => {
                let Some(sb) = self.slots.get(&slot) else {
                    return;
                };
                (sb.buffer.clone(), sb.width, sb.height, None)
            }
            DisplaySource::Shm(frame) => {
                let Some(shm) = &self.shm else {
                    println!("wl_shm not available");
                    return;
                };
                let borrowed_fd = unsafe { BorrowedFd::borrow_raw(frame.fd) };
                let pool_size = (frame.stride as i32) * (frame.height as i32);
                let pool = shm.create_pool(borrowed_fd, pool_size, qh, GlobalData);
                let buffer = pool.create_buffer(
                    0,
                    frame.width as i32,
                    frame.height as i32,
                    frame.stride as i32,
                    wl_shm::Format::Xrgb8888,
                    qh,
                    GlobalData,
                );
                if let Some(old_pool) = self.current_shm_pool.replace(pool) {
                    old_pool.destroy();
                }
                (buffer.clone(), frame.width, frame.height, Some(buffer))
            }
        };

        let surface = layer.wl_surface();
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, width as i32, height as i32);
        if let Some(viewport) = &self.viewport {
            viewport.set_destination(self.width as i32, self.height as i32);
        }
        surface.frame(qh, FrameCallbackData(surface.clone()));
        surface.commit();

        if let Some(old) = std::mem::replace(&mut self.current_shm_buffer, next_shm_buffer) {
            old.destroy();
        }

        let now = Instant::now();
        self.display_times.push_back(now);
        while let Some(&front) = self.display_times.front() {
            if now.duration_since(front) > DEBUG_STATS_WINDOW {
                self.display_times.pop_front();
            } else {
                break;
            }
        }
    }

    fn set_debug_enabled(&mut self, qh: &QueueHandle<Self>, enabled: bool) {
        self.debug_enabled = enabled;
        if enabled {
            self.ensure_debug_surface(qh);
            self.last_debug_draw = None;
        } else if let Some(surface) = &self.debug_surface {
            surface.attach(None, 0, 0);
            surface.commit();
        }
        println!("[ctl] debug overlay -> {enabled}");
    }

    fn ensure_debug_surface(&mut self, qh: &QueueHandle<Self>) {
        if self.debug_surface.is_some() {
            return;
        }
        let Some(subcompositor) = &self.subcompositor else {
            println!("[debug] wl_subcompositor not available, cannot create overlay");
            return;
        };
        let Some(layer) = &self.layer else {
            return;
        };

        let surface = self.compositor.create_surface(qh);
        let subsurface = subcompositor.get_subsurface(&surface, layer.wl_surface(), qh, GlobalData);
        let y = ((self.height as i32 - DEBUG_OVERLAY_HEIGHT) / 2).max(0);
        subsurface.set_position(12, y);
        subsurface.set_desync();
        surface.commit();

        self.debug_surface = Some(surface);
        self.debug_subsurface = Some(subsurface);
    }

    fn maybe_redraw_debug(&mut self, qh: &QueueHandle<Self>) {
        let now = Instant::now();
        if let Some(last) = self.last_debug_draw {
            if now.duration_since(last) < DEBUG_REDRAW_INTERVAL {
                return;
            }
        }
        self.last_debug_draw = Some(now);
        self.draw_debug_overlay(qh);
    }

    fn draw_debug_overlay(&mut self, qh: &QueueHandle<Self>) {
        self.ensure_debug_surface(qh);
        let (Some(surface), Some(shm)) = (self.debug_surface.clone(), &self.shm) else {
            return;
        };

        let now = Instant::now();
        let display_fps = self
            .display_times
            .iter()
            .filter(|&&t| now.duration_since(t) <= Duration::from_secs(1))
            .count();
        let capture_fps = self
            .capture_times
            .iter()
            .filter(|&&t| now.duration_since(t) <= Duration::from_secs(1))
            .count();

        let mut deltas_ms: Vec<f64> = Vec::new();
        for pair in self.display_times.iter().collect::<Vec<_>>().windows(2) {
            let dt = pair[1].duration_since(*pair[0]).as_secs_f64() * 1000.0;
            deltas_ms.push(dt);
        }
        let last_ms = deltas_ms.last().copied().unwrap_or(0.0);
        let peak_ms = deltas_ms.iter().copied().fold(0.0_f64, f64::max);
        let sparkline: Vec<f64> = deltas_ms
            .iter()
            .rev()
            .take(DEBUG_SPARKLINE_SAMPLES)
            .rev()
            .copied()
            .collect();

        let width = DEBUG_OVERLAY_WIDTH as usize;
        let height = DEBUG_OVERLAY_HEIGHT as usize;
        let stride = width * 4;
        let mut pixels = vec![0u8; stride * height];

        let (bw, bh) = (width as i32, height as i32);
        debug_paint::fill_bg(&mut pixels, stride, width, height);
        debug_paint::draw_row(
            &mut pixels,
            stride,
            bw,
            bh,
            10,
            8,
            'D',
            display_fps as u32,
            None,
        );
        debug_paint::draw_row(
            &mut pixels,
            stride,
            bw,
            bh,
            10,
            40,
            'C',
            capture_fps as u32,
            None,
        );
        debug_paint::draw_row(
            &mut pixels,
            stride,
            bw,
            bh,
            10,
            72,
            'F',
            last_ms.round() as u32,
            Some(peak_ms.round() as u32),
        );
        debug_paint::draw_sparkline(
            &mut pixels,
            stride,
            bw,
            bh,
            10,
            100,
            width - 20,
            14,
            &sparkline,
            33.3,
        );

        let memfd = unsafe { libc::memfd_create(c"wallpiper-debug-overlay".as_ptr(), 0) };
        if memfd < 0 {
            println!("[debug] memfd_create failed");
            return;
        }
        {
            let mut file = unsafe { std::fs::File::from_raw_fd(memfd) };
            let write_ok = file.write_all(&pixels).is_ok();
            std::mem::forget(file);
            if !write_ok {
                println!("[debug] failed to write overlay pixels");
                unsafe { libc::close(memfd) };
                return;
            }
        }

        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(memfd) };
        let pool = shm.create_pool(borrowed_fd, (stride * height) as i32, qh, GlobalData);
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Argb8888,
            qh,
            GlobalData,
        );

        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, width as i32, height as i32);
        surface.commit();

        if let Some(old_pool) = self.debug_shm_pool.replace(pool) {
            old_pool.destroy();
        }
        if let Some(old_buffer) = self.debug_shm_buffer.replace(buffer) {
            old_buffer.destroy();
        }
        // The compositor won't actually see this request (and mmap the fd) until the next
        // conn.flush() in the main loop, so we can't close memfd here - only the *previous*
        // redraw's fd, which has had a full cycle to be flushed and processed by now.
        if let Some(old_fd) = self.debug_shm_fd.replace(memfd) {
            unsafe { libc::close(old_fd) };
        }
    }
}

impl CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }
    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        self.refresh_buffer(qh);
    }
    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for State {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        std::process::exit(0);
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }
        println!("configure: {}x{}", self.width, self.height);
    }
}

impl DmabufHandler for State {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }
    fn dmabuf_feedback(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wayland_protocols::wp::linux_dmabuf::zv1::client::zwp_linux_dmabuf_feedback_v1::ZwpLinuxDmabufFeedbackV1,
        _: DmabufFeedback,
    ) {
    }
    fn created(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1,
        _: wl_buffer::WlBuffer,
    ) {
    }
    fn failed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1,
    ) {
        println!("dmabuf buffer creation FAILED");
    }
    fn released(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_buffer::WlBuffer) {}
}

impl Dispatch<WpViewporter, GlobalData> for State {
    fn event(
        _: &mut Self,
        _: &WpViewporter,
        _: <WpViewporter as wayland_client::Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WpViewport, GlobalData> for State {
    fn event(
        _: &mut Self,
        _: &WpViewport,
        _: <WpViewport as wayland_client::Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm::WlShm, GlobalData> for State {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        _: <wl_shm::WlShm as wayland_client::Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, GlobalData> for State {
    fn event(
        _: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        _: <wl_shm_pool::WlShmPool as wayland_client::Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, GlobalData> for State {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        _: <wl_buffer::WlBuffer as wayland_client::Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_subsurface::WlSubsurface, GlobalData> for State {
    fn event(
        _: &mut Self,
        _: &wl_subsurface::WlSubsurface,
        _: <wl_subsurface::WlSubsurface as wayland_client::Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_registry!(State);
impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

smithay_client_toolkit::delegate_dispatch2!(State);

/// Tiny self-contained ARGB8888 software rasterizer for the "stats for nerds" overlay.
/// No font/graphics libraries: digits are drawn as classic 7-segment shapes and the handful
/// of row-label letters (D/C/F) as a hand-rolled 5x5 bitmap font.
mod debug_paint {
    type Rgba = [u8; 4];

    const BG: Rgba = [20, 20, 24, 200];
    const PRIMARY: Rgba = [255, 255, 255, 255];
    const SECONDARY: Rgba = [150, 150, 150, 255];
    const BAR_OK: Rgba = [130, 190, 255, 255];
    const BAR_WARN: Rgba = [255, 90, 70, 255];

    fn put_pixel(
        pixels: &mut [u8],
        stride: usize,
        buf_w: i32,
        buf_h: i32,
        x: i32,
        y: i32,
        color: Rgba,
    ) {
        if x < 0 || y < 0 || x >= buf_w || y >= buf_h {
            return;
        }
        let offset = y as usize * stride + x as usize * 4;
        // wl_shm::Format::Argb8888 is native-endian 0xAARRGGBB - byte order [B, G, R, A] on
        // little-endian x86/arm64, which is what this whole project already targets.
        pixels[offset] = color[2];
        pixels[offset + 1] = color[1];
        pixels[offset + 2] = color[0];
        pixels[offset + 3] = color[3];
    }

    #[allow(clippy::too_many_arguments)]
    fn fill_rect(
        pixels: &mut [u8],
        stride: usize,
        buf_w: i32,
        buf_h: i32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Rgba,
    ) {
        for row in y..y + h {
            for col in x..x + w {
                put_pixel(pixels, stride, buf_w, buf_h, col, row, color);
            }
        }
    }

    pub fn fill_bg(pixels: &mut [u8], stride: usize, buf_w: usize, buf_h: usize) {
        fill_rect(
            pixels,
            stride,
            buf_w as i32,
            buf_h as i32,
            0,
            0,
            buf_w as i32,
            buf_h as i32,
            BG,
        );
    }

    // Standard 7-segment encoding, bit0=a(top) 1=b(top-right) 2=c(bottom-right) 3=d(bottom)
    // 4=e(bottom-left) 5=f(top-left) 6=g(middle).
    const SEGMENTS: [u8; 10] = [
        0b0111111, // 0
        0b0000110, // 1
        0b1011011, // 2
        0b1001111, // 3
        0b1100110, // 4
        0b1101101, // 5
        0b1111101, // 6
        0b0000111, // 7
        0b1111111, // 8
        0b1101111, // 9
    ];

    #[allow(clippy::too_many_arguments)]
    fn draw_digit(
        pixels: &mut [u8],
        stride: usize,
        buf_w: i32,
        buf_h: i32,
        x: i32,
        y: i32,
        digit: u8,
        cell_w: i32,
        cell_h: i32,
        thick: i32,
        color: Rgba,
    ) {
        let bits = SEGMENTS[(digit % 10) as usize];
        let half = cell_h / 2;
        if bits & 0b0000001 != 0 {
            fill_rect(pixels, stride, buf_w, buf_h, x, y, cell_w, thick, color);
            // a
        }
        if bits & 0b0000010 != 0 {
            fill_rect(
                pixels,
                stride,
                buf_w,
                buf_h,
                x + cell_w - thick,
                y,
                thick,
                half,
                color,
            ); // b
        }
        if bits & 0b0000100 != 0 {
            fill_rect(
                pixels,
                stride,
                buf_w,
                buf_h,
                x + cell_w - thick,
                y + half,
                thick,
                half,
                color,
            ); // c
        }
        if bits & 0b0001000 != 0 {
            fill_rect(
                pixels,
                stride,
                buf_w,
                buf_h,
                x,
                y + cell_h - thick,
                cell_w,
                thick,
                color,
            ); // d
        }
        if bits & 0b0010000 != 0 {
            fill_rect(
                pixels,
                stride,
                buf_w,
                buf_h,
                x,
                y + half,
                thick,
                half,
                color,
            ); // e
        }
        if bits & 0b0100000 != 0 {
            fill_rect(pixels, stride, buf_w, buf_h, x, y, thick, half, color); // f
        }
        if bits & 0b1000000 != 0 {
            fill_rect(
                pixels,
                stride,
                buf_w,
                buf_h,
                x,
                y + half - thick / 2,
                cell_w,
                thick,
                color,
            ); // g
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_number(
        pixels: &mut [u8],
        stride: usize,
        buf_w: i32,
        buf_h: i32,
        x: i32,
        y: i32,
        value: u32,
        cell_w: i32,
        cell_h: i32,
        thick: i32,
        color: Rgba,
    ) -> i32 {
        let value = value.min(999);
        let digits: Vec<u8> = if value == 0 {
            vec![0]
        } else {
            let mut v = value;
            let mut ds = Vec::new();
            while v > 0 {
                ds.push((v % 10) as u8);
                v /= 10;
            }
            ds.reverse();
            ds
        };
        let gap = thick;
        let mut cursor = x;
        for d in digits {
            draw_digit(
                pixels, stride, buf_w, buf_h, cursor, y, d, cell_w, cell_h, thick, color,
            );
            cursor += cell_w + gap;
        }
        cursor - x
    }

    // 5x5 bitmap font, only the glyphs this overlay actually uses. Bit4 = leftmost column.
    fn glyph_5x5(ch: char) -> Option<[u8; 5]> {
        match ch {
            'D' => Some([0b11100, 0b10010, 0b10001, 0b10010, 0b11100]),
            'C' => Some([0b01111, 0b10000, 0b10000, 0b10000, 0b01111]),
            'F' => Some([0b11111, 0b10000, 0b11110, 0b10000, 0b10000]),
            _ => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_letter(
        pixels: &mut [u8],
        stride: usize,
        buf_w: i32,
        buf_h: i32,
        x: i32,
        y: i32,
        ch: char,
        scale: i32,
        color: Rgba,
    ) {
        let Some(rows) = glyph_5x5(ch) else {
            return;
        };
        for (row_idx, row_bits) in rows.iter().enumerate() {
            for col_idx in 0..5 {
                if row_bits & (1 << (4 - col_idx)) != 0 {
                    fill_rect(
                        pixels,
                        stride,
                        buf_w,
                        buf_h,
                        x + col_idx * scale,
                        y + row_idx as i32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_row(
        pixels: &mut [u8],
        stride: usize,
        buf_w: i32,
        buf_h: i32,
        x: i32,
        y: i32,
        label: char,
        primary: u32,
        secondary: Option<u32>,
    ) {
        draw_letter(pixels, stride, buf_w, buf_h, x, y + 2, label, 3, PRIMARY);
        let number_x = x + 22;
        let consumed = draw_number(
            pixels, stride, buf_w, buf_h, number_x, y, primary, 12, 20, 3, PRIMARY,
        );
        if let Some(secondary) = secondary {
            draw_number(
                pixels,
                stride,
                buf_w,
                buf_h,
                number_x + consumed + 10,
                y + 7,
                secondary,
                8,
                13,
                2,
                SECONDARY,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_sparkline(
        pixels: &mut [u8],
        stride: usize,
        buf_w: i32,
        buf_h: i32,
        x: i32,
        y: i32,
        w: usize,
        h: i32,
        values: &[f64],
        warn_threshold_ms: f64,
    ) {
        const SAMPLES: usize = 40;
        const MAX_SCALE_MS: f64 = 40.0;

        let slot_w = ((w / SAMPLES).max(1)) as i32;
        for (i, &delta_ms) in values.iter().enumerate() {
            let bar_h = ((delta_ms / MAX_SCALE_MS).clamp(0.02, 1.0) * h as f64) as i32;
            let color = if delta_ms > warn_threshold_ms {
                BAR_WARN
            } else {
                BAR_OK
            };
            let bar_x = x + i as i32 * slot_w;
            fill_rect(
                pixels,
                stride,
                buf_w,
                buf_h,
                bar_x,
                y + h - bar_h,
                (slot_w - 1).max(1),
                bar_h,
                color,
            );
        }
    }
}
