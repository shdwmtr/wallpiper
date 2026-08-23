/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2026 Ethan Alexander
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#include "state_internal.h"

#include <stdio.h>
#include <stdlib.h>

#define WP_WL_KEYBOARD_INTERACTIVITY_NONE 0

static void layer_surface_configure(void *data,
                                    struct zwlr_layer_surface_v1 *surface,
                                    uint32_t serial, uint32_t width,
                                    uint32_t height) {
  wp_wl_output_t *out = data;
  zwlr_layer_surface_v1_ack_configure(surface, serial);

  if (width > 0) {
    out->configured_width = width;
  }
  if (height > 0) {
    out->configured_height = height;
  }
  printf("configure: output=%p %ux%u\n", (void *)out->output,
         out->configured_width, out->configured_height);
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
  (void)data;
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

void wp_wl_output_ready(wp_wl_output_t *out) {
  if (out->surface) {
    return; /* already created */
  }
  wp_wl_state_t *state = out->state;

  out->surface = wl_compositor_create_surface(state->compositor);
  wl_surface_add_listener(out->surface, &SURFACE_LISTENER, out);
  if (state->viewporter) {
    out->viewport = wp_viewporter_get_viewport(state->viewporter, out->surface);
  }

  out->layer_surface = zwlr_layer_shell_v1_get_layer_surface(
      state->layer_shell, out->surface, out->output,
      ZWLR_LAYER_SHELL_V1_LAYER_BACKGROUND, state->config->layer_namespace);
  zwlr_layer_surface_v1_add_listener(out->layer_surface,
                                     &LAYER_SURFACE_LISTENER, out);
  zwlr_layer_surface_v1_set_anchor(out->layer_surface,
                                   ZWLR_LAYER_SURFACE_V1_ANCHOR_TOP |
                                       ZWLR_LAYER_SURFACE_V1_ANCHOR_BOTTOM |
                                       ZWLR_LAYER_SURFACE_V1_ANCHOR_LEFT |
                                       ZWLR_LAYER_SURFACE_V1_ANCHOR_RIGHT);
  zwlr_layer_surface_v1_set_exclusive_zone(out->layer_surface, -1);
  zwlr_layer_surface_v1_set_keyboard_interactivity(
      out->layer_surface, WP_WL_KEYBOARD_INTERACTIVITY_NONE);
  zwlr_layer_surface_v1_set_size(out->layer_surface, 0, 0);
  wl_surface_commit(out->surface);

  printf("layer surface created for output=%p at (%d,%d) %ux%u\n",
         (void *)out->output, out->x, out->y, out->width, out->height);
}

static void output_geometry(void *data, struct wl_output *output, int32_t x,
                            int32_t y, int32_t physical_width,
                            int32_t physical_height, int32_t subpixel,
                            const char *make, const char *model,
                            int32_t transform) {
  (void)output;
  (void)physical_width;
  (void)physical_height;
  (void)subpixel;
  (void)make;
  (void)model;
  (void)transform;
  wp_wl_output_t *out = data;
  out->x = x;
  out->y = y;
}

static void output_mode(void *data, struct wl_output *output, uint32_t flags,
                        int32_t width, int32_t height, int32_t refresh) {
  (void)output;
  (void)refresh;
  if (!(flags & WL_OUTPUT_MODE_CURRENT)) {
    return;
  }
  wp_wl_output_t *out = data;
  out->width = (uint32_t)width;
  out->height = (uint32_t)height;
}

static void output_done(void *data, struct wl_output *output) {
  (void)output;
  wp_wl_output_t *out = data;
  out->known = true;
  wp_wl_output_ready(out);
  wp_wl_refresh_geometry(out->state);
}

static void output_scale(void *data, struct wl_output *output, int32_t factor) {
  (void)output;
  wp_wl_output_t *out = data;
  out->scale = factor > 0 ? (double)factor : 1.0;
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

void wp_wl_attach_output_listener(wp_wl_output_t *out) {
  wl_output_add_listener(out->output, &OUTPUT_LISTENER, out);
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

void wp_wl_request_frame_callback(wp_wl_output_t *out) {
  struct wl_callback *callback = wl_surface_frame(out->surface);
  wl_callback_add_listener(callback, &FRAME_CALLBACK_LISTENER, out);
}
