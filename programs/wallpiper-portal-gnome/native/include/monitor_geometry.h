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
#include <glib.h>
#include <meta/meta-backend.h>

G_BEGIN_DECLS
typedef struct {
  int x;
  int y;
  guint32 width;
  guint32 height;
  guint32 logical_width;
  guint32 logical_height;
  gdouble scale;
} WallpiperMonitorGeometry;

WallpiperMonitorGeometry wallpiper_monitor_detect_primary(MetaBackend *backend);
void wallpiper_monitor_detect_all(MetaBackend *backend,
                                  WallpiperMonitorGeometry *out, guint max,
                                  guint *out_count);

void wallpiper_monitor_detect_all_named(MetaBackend *backend,
                                        WallpiperMonitorGeometry *out,
                                        char *connector_names_out,
                                        size_t connector_name_len, guint max,
                                        guint *out_count);

gboolean wallpiper_x11_output_for_size(guint32 width, guint32 height,
                                       char *name_out, size_t name_out_len);

gboolean wallpiper_x11_output_for_position(gint32 x, gint32 y, char *name_out,
                                           size_t name_out_len);

gchar *
wallpiper_monitor_geometry_to_json(const WallpiperMonitorGeometry *geometry);
G_END_DECLS
