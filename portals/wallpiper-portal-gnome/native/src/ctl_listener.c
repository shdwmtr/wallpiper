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

#include "ctl_listener.h"
#include "actor_stacking.h"
#include "capture_listener.h"
#include "error.h"
#include "monitor_geometry.h"

#include <glib-unix.h>
#include <graphene.h>
#include <meta/meta-cursor-tracker.h>

#include <errno.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

static gboolean on_ctl_socket_connectable(gint fd, GIOCondition condition,
                                          gpointer user_data) {
  WallpiperPortalState *state = user_data;

  if (condition & (G_IO_ERR | G_IO_HUP)) {
    g_warning("wallpiper-gnome: ctl socket error/hangup");
    return G_SOURCE_REMOVE;
  }

  int client_fd = accept(fd, NULL, NULL);
  if (client_fd < 0)
    return G_SOURCE_CONTINUE;

  char buf[256];
  ssize_t n = recv(client_fd, buf, sizeof(buf) - 1, 0);
  if (n <= 0) {
    close(client_fd);
    return G_SOURCE_CONTINUE;
  }
  buf[n] = '\0';

  const gchar *line = g_strstrip(buf);
  g_message("wallpiper-gnome: ctl request: %s", line);
  gchar *response;

  if (g_strcmp0(line, "GEOMETRY") == 0) {
    gchar *json = wallpiper_monitor_geometry_to_json(&state->geometry);
    response = g_strdup_printf("GEOMETRY %s\n", json);
    g_free(json);
  } else if (g_strcmp0(line, "DETACH") == 0) {
    wallpiper_capture_listener_detach(state);
    response = g_strdup("OK\n");
  } else if (g_strcmp0(line, "DEBUG_ON") == 0) {
    state->debug_enabled = TRUE;
    g_message("wallpiper-gnome: debug enabled");
    wallpiper_actor_stacking_dump_children(state->parent, "debug-on");
    {
      for (int ch = 0; ch < WP_MAX_CAPTURE_CHANNELS; ch++) {
        ClutterActor *actor = state->channels[ch].display_actor;
        if (!actor)
          continue;
        guint idx = 0;
        gboolean found = FALSE;
        for (ClutterActor *child = clutter_actor_get_first_child(state->parent);
             child != NULL;
             child = clutter_actor_get_next_sibling(child), idx++) {
          if (child == actor) {
            g_message("wallpiper-gnome: [debug-on] channel %d display_actor "
                      "is at index %u",
                      ch, idx);
            found = TRUE;
            break;
          }
        }
        if (!found)
          g_message("wallpiper-gnome: [debug-on] channel %d display_actor "
                    "NOT FOUND among window_group children!",
                    ch);
      }
    }
    response = g_strdup("OK\n");
  } else if (g_strcmp0(line, "DEBUG_OFF") == 0) {
    state->debug_enabled = FALSE;
    g_message("wallpiper-gnome: debug disabled");
    response = g_strdup("OK\n");
  } else if (g_strcmp0(line, "CURSOR_POS") == 0) {
    MetaCursorTracker *tracker =
        meta_backend_get_cursor_tracker(state->backend);
    graphene_point_t point = GRAPHENE_POINT_INIT(0.f, 0.f);
    if (tracker)
      meta_cursor_tracker_get_pointer(tracker, &point, NULL);
    response =
        g_strdup_printf("CURSOR_POS %d %d\n", (int)point.x, (int)point.y);
  } else {
    response = g_strdup_printf("ERR unrecognized command\n");
  }

  send(client_fd, response, strlen(response), 0);
  g_free(response);
  close(client_fd);

  return G_SOURCE_CONTINUE;
}

gboolean wallpiper_ctl_listener_start(WallpiperPortalState *state,
                                      GError **error) {
  int ctl_fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0);
  if (ctl_fd < 0) {
    g_set_error(error, WALLPIPER_ERROR, 0, "ctl socket() failed: %s",
                g_strerror(errno));
    return FALSE;
  }

  unlink(WALLPIPER_CTL_SOCKET_PATH);

  struct sockaddr_un addr;
  memset(&addr, 0, sizeof(addr));
  addr.sun_family = AF_UNIX;
  g_strlcpy(addr.sun_path, WALLPIPER_CTL_SOCKET_PATH, sizeof(addr.sun_path));
  if (bind(ctl_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0 ||
      listen(ctl_fd, 4) != 0) {
    g_set_error(error, WALLPIPER_ERROR, 0, "bind/listen(%s) failed: %s",
                WALLPIPER_CTL_SOCKET_PATH, g_strerror(errno));
    close(ctl_fd);
    return FALSE;
  }

  state->ctl_socket_fd = ctl_fd;
  state->ctl_source_id =
      g_unix_fd_add(ctl_fd, G_IO_IN, on_ctl_socket_connectable, state);

  return TRUE;
}

void wallpiper_ctl_listener_stop(WallpiperPortalState *state) {
  g_source_remove(state->ctl_source_id);
  close(state->ctl_socket_fd);
  unlink(WALLPIPER_CTL_SOCKET_PATH);
}
