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

#pragma once

#include <stdbool.h>
#include <stdint.h>

#define WP_CAPTURE_MAX_FDS 2

typedef enum {
  WP_CAPTURE_EVENT_BUF,
  WP_CAPTURE_EVENT_FRAME,
  WP_CAPTURE_EVENT_SHM,
} wp_capture_event_tag_t;

typedef struct {
  wp_capture_event_tag_t tag;
  uint32_t slot;
  uint32_t width;
  uint32_t height;
  uint32_t stride;
  uint64_t modifier;
  bool has_geometry;
  int32_t geom_x;
  int32_t geom_y;
  int fds[WP_CAPTURE_MAX_FDS];
  int nfds;
} wp_capture_event_t;

int wp_bind_capture_socket(void);
bool wp_recv_capture_event(int sock_fd, wp_capture_event_t *out);

typedef struct wp_capture_link wp_capture_link_t;

wp_capture_link_t *wp_capture_link_create(void);
void wp_capture_link_destroy(wp_capture_link_t *link);

bool wp_capture_link_send_buf(wp_capture_link_t *link, uint32_t slot,
                              uint32_t width, uint32_t height,
                              uint32_t format_raw, uint32_t stride,
                              uint64_t modifier, bool has_geometry,
                              int32_t geom_x, int32_t geom_y, int image_fd,
                              int sync_fd);
bool wp_capture_link_send_frame(wp_capture_link_t *link, uint32_t slot,
                                int sync_fd);
