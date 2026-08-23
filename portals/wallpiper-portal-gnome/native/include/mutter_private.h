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
#include <meta/meta-backend.h>
#include <meta/meta-logical-monitor.h>
#include <meta/meta-monitor-manager.h>
#include <meta/meta-monitor.h>
#include <mtk/mtk.h>

G_BEGIN_DECLS
extern ClutterBackend *meta_backend_get_clutter_backend(MetaBackend *backend);
typedef struct _MetaMonitorMode MetaMonitorMode;

extern MetaLogicalMonitor *
meta_monitor_manager_get_primary_logical_monitor(MetaMonitorManager *manager);
extern MtkRectangle
meta_logical_monitor_get_layout(MetaLogicalMonitor *logical_monitor);
extern float
meta_logical_monitor_get_scale(MetaLogicalMonitor *logical_monitor);
extern MetaMonitorMode *meta_monitor_get_current_mode(MetaMonitor *monitor);
extern void meta_monitor_mode_get_resolution(MetaMonitorMode *mode, int *width,
                                             int *height);
G_END_DECLS
