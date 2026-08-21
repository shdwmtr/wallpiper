#include "state_internal.h"

#include <math.h>
#include <poll.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define WP_WL_KEYBOARD_INTERACTIVITY_NONE 0

void wp_wl_geometry_from_scale(int32_t x, int32_t y, uint32_t width,
                               uint32_t height, double scale,
                               wp_monitor_geometry_t *out) {
  if (scale <= 0.0) {
    scale = 1.0;
  }
  out->x = x;
  out->y = y;
  out->width = width;
  out->height = height;
  out->logical_width = (uint32_t)llround((double)width / scale);
  out->logical_height = (uint32_t)llround((double)height / scale);
  out->scale = scale;
}

bool wp_wl_detect_geometry(wp_wl_try_geometry_fn try_fn,
                           wp_monitor_geometry_t *out) {
  for (int attempt = 1; attempt <= 3; attempt++) {
    if (try_fn(out)) {
      return true;
    }
    printf("monitor detection attempt %d failed, retrying\n", attempt);
    struct timespec ts = {.tv_sec = 0, .tv_nsec = 500 * 1000 * 1000};
    nanosleep(&ts, NULL);
  }
  printf("monitor detection failed after retries, falling back to 1920x1080 at "
         "0,0\n");
  out->x = 0;
  out->y = 0;
  out->width = 1920;
  out->height = 1080;
  out->logical_width = 1920;
  out->logical_height = 1080;
  out->scale = 1.0;
  return true;
}

static void handle_ctl_request(wp_wl_state_t *state, wp_ctl_request_t request) {
  wp_ctl_response_t response;
  memset(&response, 0, sizeof(response));

  switch (request) {
  case WP_CTL_REQUEST_GEOMETRY:
    response.tag = WP_CTL_RESPONSE_GEOMETRY;
    response.geometry = state->geometry;
    break;
  case WP_CTL_REQUEST_DETACH:
    wp_wl_detach(state);
    response.tag = WP_CTL_RESPONSE_OK;
    break;
  case WP_CTL_REQUEST_DEBUG_ON:
    wp_wl_set_debug_enabled(state, true);
    response.tag = WP_CTL_RESPONSE_OK;
    break;
  case WP_CTL_REQUEST_DEBUG_OFF:
    wp_wl_set_debug_enabled(state, false);
    response.tag = WP_CTL_RESPONSE_OK;
    break;
  case WP_CTL_REQUEST_CURSOR_POS:
    response.tag = WP_CTL_RESPONSE_ERR;
    snprintf(response.err, sizeof(response.err), "%s",
             "handled by ctl listener");
    break;
  default:
    response.tag = WP_CTL_RESPONSE_ERR;
    snprintf(response.err, sizeof(response.err), "%s", "unrecognized command");
    break;
  }

  wp_ctl_listener_reply(state->ctl_listener, &response);
}

void wp_wl_portal_run(const wp_wl_portal_config_t *config) {
  setvbuf(stdout, NULL, _IOLBF, 0);

  wp_wl_state_t state;
  memset(&state, 0, sizeof(state));
  state.config = config;
  state.width = 1920;
  state.height = 1080;
  state.debug_shm_fd = -1;
  state.stats = wp_frame_stats_create();
  wp_debug_throttle_init(&state.debug_throttle);

  wp_wl_detect_geometry(config->try_geometry, &state.geometry);
  printf(
      "detected monitor geometry: x=%d y=%d %ux%u logical=%ux%u scale=%.4f\n",
      state.geometry.x, state.geometry.y, state.geometry.width,
      state.geometry.height, state.geometry.logical_width,
      state.geometry.logical_height, state.geometry.scale);

  state.display = wl_display_connect(NULL);
  if (!state.display) {
    fprintf(stderr, "connect to wayland: failed\n");
    exit(1);
  }

  wp_wl_bind_globals(&state);

  state.surface = wl_compositor_create_surface(state.compositor);
  wp_wl_attach_surface_listener(&state);
  if (state.viewporter) {
    state.viewport =
        wp_viewporter_get_viewport(state.viewporter, state.surface);
  }

  state.layer_surface = zwlr_layer_shell_v1_get_layer_surface(
      state.layer_shell, state.surface, NULL,
      ZWLR_LAYER_SHELL_V1_LAYER_BACKGROUND, config->layer_namespace);
  wp_wl_attach_layer_surface_listener(&state);
  zwlr_layer_surface_v1_set_anchor(state.layer_surface,
                                   ZWLR_LAYER_SURFACE_V1_ANCHOR_TOP |
                                       ZWLR_LAYER_SURFACE_V1_ANCHOR_BOTTOM |
                                       ZWLR_LAYER_SURFACE_V1_ANCHOR_LEFT |
                                       ZWLR_LAYER_SURFACE_V1_ANCHOR_RIGHT);
  zwlr_layer_surface_v1_set_exclusive_zone(state.layer_surface, -1);
  zwlr_layer_surface_v1_set_keyboard_interactivity(
      state.layer_surface, WP_WL_KEYBOARD_INTERACTIVITY_NONE);
  zwlr_layer_surface_v1_set_size(state.layer_surface, 0, 0);
  wl_surface_commit(state.surface);

  printf("layer surface created, waiting for configure + frame\n");

  state.capture_fd = wp_bind_capture_socket();
  state.ctl_listener = wp_ctl_listener_start(
      config->portal_name, config->cursor_pos, config->cursor_ctx);

  for (;;) {
    wl_display_dispatch_pending(state.display);
    wl_display_flush(state.display);

    for (;;) {
      wp_capture_event_t event;
      if (!wp_recv_capture_event(state.capture_fd, &event)) {
        break;
      }
      wp_wl_handle_capture_event(&state, &event);
    }

    wp_ctl_request_t request;
    if (state.ctl_listener &&
        wp_ctl_listener_poll(state.ctl_listener, &request)) {
      handle_ctl_request(&state, request);
    }

    if (state.debug_enabled) {
      wp_wl_maybe_redraw_debug(&state);
    }

    struct pollfd pollfds[2] = {
        {.fd = state.capture_fd, .events = POLLIN, .revents = 0},
        {.fd = wl_display_get_fd(state.display),
         .events = POLLIN,
         .revents = 0},
    };
    poll(pollfds, 2, 250);

    if (pollfds[1].revents & POLLIN) {
      wl_display_dispatch(state.display);
    }
  }
}
