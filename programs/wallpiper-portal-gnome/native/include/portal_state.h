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
#include "monitor_geometry.h"
#include "protocol.h"

#include <EGL/egl.h>
#include <meta/display.h>
#include <meta/meta-backend.h>

G_BEGIN_DECLS

typedef struct {
  gboolean used;
  guint32 width;
  guint32 height;
  CoglTexture *texture;
} WallpiperCaptureSlot;

typedef struct {
  gboolean active;
  ClutterActor *display_actor;
  WallpiperCaptureSlot slots[WP_CAPTURE_SLOT_COUNT];
  WallpiperMonitorGeometry monitor;
  gint current_slot_idx;
} WallpiperCaptureChannel;

typedef struct _WallpiperPortalState {
  MetaBackend *backend;
  CoglContext *cogl_context;
  EGLDisplay egl_display;

  ClutterActor *parent;

  WallpiperCaptureChannel channels[WP_MAX_CAPTURE_CHANNELS];
  WallpiperMonitorGeometry geometry;
  gboolean debug_enabled;

  int capture_socket_fd;
  guint capture_source_id;

  int ctl_socket_fd;
  guint ctl_source_id;

  MetaDisplay *meta_display;
  gulong restacked_handler_id;
  gulong window_created_handler_id;
  gulong monitors_changed_handler_id;
} WallpiperPortalState;

G_END_DECLS
