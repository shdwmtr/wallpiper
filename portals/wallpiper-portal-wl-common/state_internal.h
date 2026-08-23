#pragma once

#include <stdbool.h>
#include <stdint.h>

#include <wayland-client.h>

#include "linux-dmabuf-v1-client-protocol.h"
#include "viewporter-client-protocol.h"
#include "wlr-layer-shell-unstable-v1-client-protocol.h"

#include "wallpiper/capture_socket.h"
#include "wallpiper/ctl_protocol.h"
#include "wallpiper/debug_overlay.h"
#include "wallpiper_wl/portal.h"

/* must match layer/siphon/config.h */
#define WP_WL_CAPTURE_SLOT_COUNT 3
#define WP_WL_MAX_CAPTURE_CHANNELS 4
#define WP_WL_MAX_SLOTS (WP_WL_MAX_CAPTURE_CHANNELS * WP_WL_CAPTURE_SLOT_COUNT)

#define WP_WL_MAX_OUTPUTS WP_WL_MAX_CAPTURE_CHANNELS
#define WP_WL_DMABUF_MIN_VERSION 2
#define WP_WL_DRM_FORMAT_XRGB8888 0x34325258u

typedef struct {
  bool in_use;
  uint32_t slot;
  struct wl_buffer *buffer;
  int fd;
  uint32_t width;
  uint32_t height;
} wp_wl_slot_t;

typedef enum {
  WP_WL_SOURCE_NONE,
  WP_WL_SOURCE_SLOT,
  WP_WL_SOURCE_SHM,
} wp_wl_source_kind_t;

typedef struct {
  wp_wl_source_kind_t kind;
  uint32_t slot;
  int fd;
  uint32_t width;
  uint32_t height;
  uint32_t stride;
} wp_wl_source_t;

typedef struct wp_wl_state wp_wl_state_t;

typedef struct {
  wp_wl_state_t *state;
  struct wl_output *output;
  bool known;
  int32_t x;
  int32_t y;
  uint32_t width; /* physical pixels, from wl_output.mode */
  uint32_t height;
  double scale;

  bool has_bound_channel;
  uint32_t bound_channel;

  struct wl_surface *surface;
  struct zwlr_layer_surface_v1 *layer_surface;
  struct wp_viewport *viewport;
  uint32_t configured_width;
  uint32_t configured_height;

  wp_wl_slot_t slots[WP_WL_CAPTURE_SLOT_COUNT];
  bool has_current_source;
  wp_wl_source_t current_source;
  struct wl_buffer *current_shm_buffer;
  struct wl_shm_pool *current_shm_pool;
  bool frame_loop_running;
} wp_wl_output_t;

struct wp_wl_state {
  const wp_wl_portal_config_t *config;

  struct wl_display *display;
  struct wl_registry *registry;
  struct wl_compositor *compositor;
  struct zwlr_layer_shell_v1 *layer_shell;
  struct zwp_linux_dmabuf_v1 *dmabuf;
  struct wp_viewporter *viewporter;
  struct wl_shm *shm;
  struct wl_subcompositor *subcompositor;

  wp_monitor_geometry_t geometry;

  wp_wl_output_t outputs[WP_WL_MAX_OUTPUTS];
  size_t output_count;

  bool debug_enabled;
  struct wl_surface *debug_surface;
  struct wl_subsurface *debug_subsurface;
  struct wl_shm_pool *debug_shm_pool;
  struct wl_buffer *debug_shm_buffer;
  bool has_debug_shm_fd;
  int debug_shm_fd;
  wp_debug_throttle_t debug_throttle;
  wp_frame_stats_t *stats;

  int capture_fd;
  wp_ctl_listener_t *ctl_listener;
};

bool wp_wl_detect_geometry(wp_wl_try_geometry_fn try_fn,
                           wp_monitor_geometry_t *out);

void wp_wl_bind_globals(wp_wl_state_t *state);
wp_wl_output_t *wp_wl_add_output(wp_wl_state_t *state,
                                 struct wl_output *output);
void wp_wl_attach_output_listener(wp_wl_output_t *out);
void wp_wl_output_ready(wp_wl_output_t *out);
void wp_wl_request_frame_callback(wp_wl_output_t *out);

void wp_wl_refresh_geometry(wp_wl_state_t *state);
void wp_wl_set_current_source(wp_wl_output_t *out, wp_wl_source_t source);
void wp_wl_handle_capture_event(wp_wl_state_t *state,
                                const wp_capture_event_t *event);
void wp_wl_detach(wp_wl_state_t *state);
void wp_wl_refresh_buffer(wp_wl_output_t *out);

void wp_wl_set_debug_enabled(wp_wl_state_t *state, bool enabled);
void wp_wl_maybe_redraw_debug(wp_wl_state_t *state);
void wp_wl_draw_debug_overlay(wp_wl_state_t *state);
void wp_wl_ensure_debug_surface(wp_wl_state_t *state);

struct wl_buffer *wp_wl_create_dmabuf_buffer(wp_wl_state_t *state, int fd,
                                             uint32_t width, uint32_t height,
                                             uint32_t stride,
                                             uint64_t modifier);
