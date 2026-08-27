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

#include "monitor_geometry.h"
#include "mutter_private.h"

#include <string.h>

#include <X11/Xlib.h>
#include <X11/extensions/Xrandr.h>

static WallpiperMonitorGeometry
geometry_from_logical_monitor(MetaLogicalMonitor *logical_monitor) {
  MtkRectangle layout = meta_logical_monitor_get_layout(logical_monitor);
  gdouble scale = (gdouble)meta_logical_monitor_get_scale(logical_monitor);
  if (scale <= 0.0) {
    scale = 1.0;
  }

  WallpiperMonitorGeometry geometry = {
      .x = layout.x,
      .y = layout.y,
      .width = (guint32)layout.width,
      .height = (guint32)layout.height,
      .logical_width = (guint32)layout.width,
      .logical_height = (guint32)layout.height,
      .scale = scale,
  };

  GList *monitors = meta_logical_monitor_get_monitors(logical_monitor);
  if (monitors) {
    MetaMonitor *monitor = monitors->data;
    MetaMonitorMode *mode = meta_monitor_get_current_mode(monitor);
    if (mode) {
      int width = 0, height = 0;
      meta_monitor_mode_get_resolution(mode, &width, &height);
      if (width > 0 && height > 0) {
        geometry.width = (guint32)width;
        geometry.height = (guint32)height;
      }
    }
  }

  return geometry;
}

WallpiperMonitorGeometry
wallpiper_monitor_detect_primary(MetaBackend *backend) {
  WallpiperMonitorGeometry fallback = {0, 0, 1920, 1080, 1920, 1080, 1.0};

  MetaMonitorManager *manager = meta_backend_get_monitor_manager(backend);
  if (!manager)
    return fallback;

  MetaLogicalMonitor *logical_monitor =
      meta_monitor_manager_get_primary_logical_monitor(manager);
  if (!logical_monitor) {
    GList *logical_monitors =
        meta_monitor_manager_get_logical_monitors(manager);
    if (!logical_monitors)
      return fallback;
    logical_monitor = logical_monitors->data;
  }

  return geometry_from_logical_monitor(logical_monitor);
}

void wallpiper_monitor_detect_all(MetaBackend *backend,
                                  WallpiperMonitorGeometry *out, guint max,
                                  guint *out_count) {
  *out_count = 0;

  MetaMonitorManager *manager = meta_backend_get_monitor_manager(backend);
  if (!manager)
    return;

  GList *logical_monitors = meta_monitor_manager_get_logical_monitors(manager);
  for (GList *l = logical_monitors; l && *out_count < max; l = l->next) {
    out[(*out_count)++] = geometry_from_logical_monitor(l->data);
  }
}

void wallpiper_monitor_detect_all_named(MetaBackend *backend,
                                        WallpiperMonitorGeometry *out,
                                        char *connector_names_out,
                                        size_t connector_name_len, guint max,
                                        guint *out_count) {
  *out_count = 0;

  MetaMonitorManager *manager = meta_backend_get_monitor_manager(backend);
  if (!manager)
    return;

  GList *logical_monitors = meta_monitor_manager_get_logical_monitors(manager);
  for (GList *l = logical_monitors; l && *out_count < max; l = l->next) {
    MetaLogicalMonitor *logical_monitor = l->data;
    out[*out_count] = geometry_from_logical_monitor(logical_monitor);

    char *name_slot = connector_names_out + (*out_count) * connector_name_len;
    name_slot[0] = '\0';
    GList *monitors = meta_logical_monitor_get_monitors(logical_monitor);
    if (monitors) {
      const char *connector = meta_monitor_get_connector(monitors->data);
      if (connector) {
        g_strlcpy(name_slot, connector, connector_name_len);
      }
    }

    (*out_count)++;
  }
}

static Display *get_x11_display(void) {
  static Display *dpy = NULL;
  static gboolean tried = FALSE;
  if (!tried) {
    tried = TRUE;
    dpy = XOpenDisplay(NULL);
    if (!dpy) {
      g_warning("wallpiper-gnome: XOpenDisplay failed, X11-space queries "
                "unavailable");
    }
  }
  return dpy;
}

gboolean wallpiper_x11_output_for_size(guint32 width, guint32 height,
                                       char *name_out, size_t name_out_len) {
  name_out[0] = '\0';

  Display *dpy = get_x11_display();
  if (!dpy) {
    return FALSE;
  }

  Window root = DefaultRootWindow(dpy);
  XRRScreenResources *res = XRRGetScreenResourcesCurrent(dpy, root);
  if (!res) {
    return FALSE;
  }

  gboolean found = FALSE;
  for (int i = 0; i < res->noutput && !found; i++) {
    XRROutputInfo *output_info = XRRGetOutputInfo(dpy, res, res->outputs[i]);
    if (!output_info) {
      continue;
    }
    if (output_info->connection == RR_Connected && output_info->crtc != None) {
      XRRCrtcInfo *crtc_info = XRRGetCrtcInfo(dpy, res, output_info->crtc);
      if (crtc_info) {
        if (crtc_info->width == width && crtc_info->height == height) {
          g_strlcpy(name_out, output_info->name, name_out_len);
          found = TRUE;
        }
        XRRFreeCrtcInfo(crtc_info);
      }
    }
    XRRFreeOutputInfo(output_info);
  }

  XRRFreeScreenResources(res);
  return found;
}

gboolean wallpiper_x11_output_for_position(gint32 x, gint32 y, char *name_out,
                                           size_t name_out_len) {
  name_out[0] = '\0';

  Display *dpy = get_x11_display();
  if (!dpy) {
    return FALSE;
  }

  Window root = DefaultRootWindow(dpy);
  XRRScreenResources *res = XRRGetScreenResourcesCurrent(dpy, root);
  if (!res) {
    return FALSE;
  }

  gboolean found = FALSE;
  for (int i = 0; i < res->noutput && !found; i++) {
    XRROutputInfo *output_info = XRRGetOutputInfo(dpy, res, res->outputs[i]);
    if (!output_info) {
      continue;
    }
    if (output_info->connection == RR_Connected && output_info->crtc != None) {
      XRRCrtcInfo *crtc_info = XRRGetCrtcInfo(dpy, res, output_info->crtc);
      if (crtc_info) {
        if (crtc_info->x == x && crtc_info->y == y) {
          g_strlcpy(name_out, output_info->name, name_out_len);
          found = TRUE;
        }
        XRRFreeCrtcInfo(crtc_info);
      }
    }
    XRRFreeOutputInfo(output_info);
  }

  XRRFreeScreenResources(res);
  return found;
}

gchar *wallpiper_monitor_geometry_to_json(const WallpiperMonitorGeometry *g) {
  return g_strdup_printf(
      "{\"x\":%d,\"y\":%d,\"width\":%u,\"height\":%u,"
      "\"logical_width\":%u,\"logical_height\":%u,\"scale\":%g}",
      g->x, g->y, g->width, g->height, g->logical_width, g->logical_height,
      g->scale);
}
