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

#include "wallpiper_portal.h"
#include "actor_stacking.h"
#include "capture_listener.h"
#include "ctl_listener.h"
#include "error.h"
#include "monitor_geometry.h"
#include "mutter_private.h"
#include "portal_state.h"
#include "protocol.h"

#include <meta/meta-context.h>

static WallpiperPortalState *portal_state = NULL;

static void on_monitors_changed(MetaMonitorManager *manager,
                                gpointer user_data) {
  (void)manager;
  WallpiperPortalState *state = (WallpiperPortalState *)user_data;
  WallpiperMonitorGeometry geometry =
      wallpiper_monitor_detect_primary(state->backend);
  state->geometry = geometry;

  g_message("wallpiper-gnome: monitor geometry changed %ux%u at (%d,%d), "
            "logical %ux%u, scale %g",
            geometry.width, geometry.height, geometry.x, geometry.y,
            geometry.logical_width, geometry.logical_height, geometry.scale);

  WallpiperMonitorGeometry monitors[WP_MAX_CAPTURE_CHANNELS];
  guint count = 0;
  wallpiper_monitor_detect_all(state->backend, monitors,
                               WP_MAX_CAPTURE_CHANNELS, &count);

  for (int i = 0; i < WP_MAX_CAPTURE_CHANNELS; i++) {
    WallpiperCaptureChannel *ch = &state->channels[i];
    if (!ch->active)
      continue;
    for (guint j = 0; j < count; j++) {
      if (monitors[j].x == ch->monitor.x && monitors[j].y == ch->monitor.y) {
        ch->monitor = monitors[j];
        clutter_actor_set_position(ch->display_actor, monitors[j].x,
                                   monitors[j].y);
        clutter_actor_set_size(ch->display_actor, monitors[j].logical_width,
                               monitors[j].logical_height);
        break;
      }
    }
  }
}

gboolean wallpiper_portal_start(GObject *backend_obj, GObject *parent_obj,
                                GError **error) {
  if (portal_state) {
    g_set_error(error, WALLPIPER_ERROR, 0, "portal already running");
    return FALSE;
  }

  if (!META_IS_BACKEND(backend_obj)) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "backend argument is not a MetaBackend");
    return FALSE;
  }

  if (!CLUTTER_IS_ACTOR(parent_obj)) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "parent argument is not a ClutterActor");
    return FALSE;
  }

  MetaBackend *backend = META_BACKEND(backend_obj);
  ClutterBackend *clutter_backend = meta_backend_get_clutter_backend(backend);
  CoglContext *cogl_context = clutter_backend_get_cogl_context(clutter_backend);
  EGLDisplay egl_display = cogl_context_get_egl_display(cogl_context);

  if (!egl_display) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "could not get Mutter's EGLDisplay from Cogl");
    return FALSE;
  }

  WallpiperMonitorGeometry geometry = wallpiper_monitor_detect_primary(backend);
  g_message("wallpiper-gnome: detected monitor geometry %ux%u at (%d,%d), "
            "logical %ux%u",
            geometry.width, geometry.height, geometry.x, geometry.y,
            geometry.logical_width, geometry.logical_height);

  WallpiperPortalState *state = g_new0(WallpiperPortalState, 1);
  state->backend = backend;
  state->cogl_context = cogl_context;
  state->egl_display = egl_display;
  state->geometry = geometry;

  ClutterActor *parent = CLUTTER_ACTOR(parent_obj);
  state->parent = parent;

  MetaMonitorManager *monitor_manager =
      meta_backend_get_monitor_manager(backend);
  if (monitor_manager) {
    state->monitors_changed_handler_id =
        g_signal_connect(monitor_manager, "monitors-changed",
                         G_CALLBACK(on_monitors_changed), state);
  }

  MetaContext *meta_context = meta_backend_get_context(backend);
  state->meta_display =
      meta_context ? meta_context_get_display(meta_context) : NULL;
  wallpiper_actor_stacking_connect(state);

  if (!wallpiper_capture_listener_start(state, error)) {
    wallpiper_actor_stacking_disconnect(state);
    g_free(state);
    return FALSE;
  }

  if (!wallpiper_ctl_listener_start(state, error)) {
    wallpiper_capture_listener_stop(state);
    wallpiper_actor_stacking_disconnect(state);
    g_free(state);
    return FALSE;
  }

  portal_state = state;

  g_message("wallpiper-gnome: portal started: capture socket %s, ctl socket %s",
            WALLPIPER_CAPTURE_SOCKET_PATH, WALLPIPER_CTL_SOCKET_PATH);

  return TRUE;
}

void wallpiper_portal_stop(void) {
  if (!portal_state)
    return;

  MetaMonitorManager *monitor_manager =
      meta_backend_get_monitor_manager(portal_state->backend);
  if (monitor_manager && portal_state->monitors_changed_handler_id) {
    g_signal_handler_disconnect(monitor_manager,
                                portal_state->monitors_changed_handler_id);
  }

  wallpiper_ctl_listener_stop(portal_state);
  wallpiper_capture_listener_stop(portal_state);
  wallpiper_actor_stacking_disconnect(portal_state);

  g_free(portal_state);
  portal_state = NULL;

  g_message("wallpiper-gnome: portal stopped");
}
