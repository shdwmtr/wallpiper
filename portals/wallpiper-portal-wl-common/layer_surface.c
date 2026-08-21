#include "state_internal.h"

#include <stdio.h>
#include <stdlib.h>

static void layer_surface_configure(void *data,
                                    struct zwlr_layer_surface_v1 *surface,
                                    uint32_t serial, uint32_t width,
                                    uint32_t height) {
  wp_wl_state_t *state = data;
  zwlr_layer_surface_v1_ack_configure(surface, serial);

  if (width > 0) {
    state->width = width;
  }
  if (height > 0) {
    state->height = height;
  }
  printf("configure: %ux%u\n", state->width, state->height);
}

static void layer_surface_closed(void *data,
                                 struct zwlr_layer_surface_v1 *surface) {
  (void)data;
  (void)surface;
  exit(0);
}

static const struct zwlr_layer_surface_v1_listener LAYER_SURFACE_LISTENER = {
    .configure = layer_surface_configure,
    .closed = layer_surface_closed,
};

void wp_wl_attach_layer_surface_listener(wp_wl_state_t *state) {
  zwlr_layer_surface_v1_add_listener(state->layer_surface,
                                     &LAYER_SURFACE_LISTENER, state);
}

static void surface_enter(void *data, struct wl_surface *surface,
                          struct wl_output *output) {
  (void)data;
  (void)surface;
  (void)output;
}

static void surface_leave(void *data, struct wl_surface *surface,
                          struct wl_output *output) {
  (void)data;
  (void)surface;
  (void)output;
}

static void surface_preferred_buffer_scale(void *data,
                                           struct wl_surface *surface,
                                           int32_t factor) {
  (void)surface;
  (void)factor;
  wp_wl_refresh_geometry(data);
}

static void surface_preferred_buffer_transform(void *data,
                                               struct wl_surface *surface,
                                               uint32_t transform) {
  (void)data;
  (void)surface;
  (void)transform;
}

static const struct wl_surface_listener SURFACE_LISTENER = {
    .enter = surface_enter,
    .leave = surface_leave,
    .preferred_buffer_scale = surface_preferred_buffer_scale,
    .preferred_buffer_transform = surface_preferred_buffer_transform,
};

void wp_wl_attach_surface_listener(wp_wl_state_t *state) {
  wl_surface_add_listener(state->surface, &SURFACE_LISTENER, state);
}

static void output_geometry(void *data, struct wl_output *output, int32_t x,
                            int32_t y, int32_t physical_width,
                            int32_t physical_height, int32_t subpixel,
                            const char *make, const char *model,
                            int32_t transform) {
  (void)data;
  (void)output;
  (void)x;
  (void)y;
  (void)physical_width;
  (void)physical_height;
  (void)subpixel;
  (void)make;
  (void)model;
  (void)transform;
}

static void output_mode(void *data, struct wl_output *output, uint32_t flags,
                        int32_t width, int32_t height, int32_t refresh) {
  (void)data;
  (void)output;
  (void)flags;
  (void)width;
  (void)height;
  (void)refresh;
}

static void output_done(void *data, struct wl_output *output) {
  (void)output;
  wp_wl_refresh_geometry(data);
}

static void output_scale(void *data, struct wl_output *output, int32_t factor) {
  (void)data;
  (void)output;
  (void)factor;
}

static void output_name(void *data, struct wl_output *output,
                        const char *name) {
  (void)data;
  (void)output;
  (void)name;
}

static void output_description(void *data, struct wl_output *output,
                               const char *description) {
  (void)data;
  (void)output;
  (void)description;
}

static const struct wl_output_listener OUTPUT_LISTENER = {
    .geometry = output_geometry,
    .mode = output_mode,
    .done = output_done,
    .scale = output_scale,
    .name = output_name,
    .description = output_description,
};

void wp_wl_attach_output_listener(wp_wl_state_t *state,
                                  struct wl_output *output) {
  wl_output_add_listener(output, &OUTPUT_LISTENER, state);
}

static void frame_callback_done(void *data, struct wl_callback *callback,
                                uint32_t callback_data) {
  (void)callback_data;
  wl_callback_destroy(callback);
  wp_wl_refresh_buffer(data);
}

static const struct wl_callback_listener FRAME_CALLBACK_LISTENER = {
    .done = frame_callback_done,
};

void wp_wl_request_frame_callback(wp_wl_state_t *state) {
  struct wl_callback *callback = wl_surface_frame(state->surface);
  wl_callback_add_listener(callback, &FRAME_CALLBACK_LISTENER, state);
}
