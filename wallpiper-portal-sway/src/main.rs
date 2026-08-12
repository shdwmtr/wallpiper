use std::collections::HashMap;
use std::io::Write;
use std::os::unix::io::{BorrowedFd, FromRawFd, RawFd};
use std::process::Command;
use std::time::Duration;

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

use wallpiper_protocol::debug_overlay as debug_paint;
use wallpiper_protocol::{CtlRequest, CtlResponse, MonitorGeometry, SocketEvent};

const DRM_FORMAT_XRGB8888: u32 = 0x34325258;

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

fn try_detect_geometry() -> Option<MonitorGeometry> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_outputs", "-r"])
        .output()
        .ok()?;
    let outputs: Value = serde_json::from_slice(&output.stdout).ok()?;
    let focused = outputs
        .as_array()?
        .iter()
        .find(|o| o["focused"] == true && o["active"] == true)?;
    let rect = &focused["rect"];
    let x = rect["x"].as_i64()? as i32;
    let y = rect["y"].as_i64()? as i32;
    let width = rect["width"].as_u64()? as u32;
    let height = rect["height"].as_u64()? as u32;
    let scale = focused["scale"].as_f64().unwrap_or(1.0);
    Some(geometry_from_scale(x, y, width, height, scale))
}

fn query_cursor_pos() -> Option<(i32, i32)> {
    None
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
        debug_throttle: debug_paint::DebugThrottle::new(),
        stats: debug_paint::FrameStats::new(),
    };

    let surface = state.compositor.create_surface(&qh);
    if let Some(viewporter) = &viewporter {
        state.viewport = Some(viewporter.get_viewport(&surface, &qh, GlobalData));
    }
    let layer = state.layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Background,
        Some("wallpiper-portal-sway"),
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
    let ctl_rx = wallpiper_protocol::spawn_ctl_listener("sway", query_cursor_pos);

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
        while let Ok(event) = event_rx.try_recv() {
            state.handle_event(&qh, event);
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
    debug_throttle: debug_paint::DebugThrottle,
    stats: debug_paint::FrameStats,
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

        self.stats.record_display();
    }

    fn set_debug_enabled(&mut self, qh: &QueueHandle<Self>, enabled: bool) {
        self.debug_enabled = enabled;
        if enabled {
            self.ensure_debug_surface(qh);
            self.debug_throttle.reset();
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
        let y = ((self.height as i32 - debug_paint::HEIGHT as i32) / 2).max(0);
        subsurface.set_position(12, y);
        subsurface.set_desync();
        surface.commit();

        self.debug_surface = Some(surface);
        self.debug_subsurface = Some(subsurface);
    }

    fn maybe_redraw_debug(&mut self, qh: &QueueHandle<Self>) {
        if self.debug_throttle.should_redraw() {
            self.draw_debug_overlay(qh);
        }
    }

    fn draw_debug_overlay(&mut self, qh: &QueueHandle<Self>) {
        self.ensure_debug_surface(qh);
        let (Some(surface), Some(shm)) = (self.debug_surface.clone(), &self.shm) else {
            return;
        };

        let pixels = debug_paint::render_stats_panel(&self.stats);
        let width = debug_paint::WIDTH as usize;
        let height = debug_paint::HEIGHT as usize;
        let stride = width * 4;

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
