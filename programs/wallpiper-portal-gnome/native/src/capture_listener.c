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

#include "capture_listener.h"
#include "actor_stacking.h"
#include "egl_import.h"
#include "error.h"
#include "monitor_geometry.h"

#include <errno.h>
#include <glib-unix.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

static void channel_clear(WallpiperCaptureChannel *ch) {
  for (int i = 0; i < WP_CAPTURE_SLOT_COUNT; i++) {
    WallpiperCaptureSlot *slot = &ch->slots[i];
    if (slot->texture) {
      g_object_unref(slot->texture);
      slot->texture = NULL;
    }
    slot->used = FALSE;
  }
  if (ch->display_actor) {
    clutter_actor_destroy(ch->display_actor);
    ch->display_actor = NULL;
  }
  ch->active = FALSE;
}

void wallpiper_capture_listener_detach(WallpiperPortalState *state) {
  for (int i = 0; i < WP_MAX_CAPTURE_CHANNELS; i++) {
    channel_clear(&state->channels[i]);
  }
  g_message("wallpiper-gnome: detached, cleared all channels");
}

static gboolean find_monitor_for_size(WallpiperPortalState *state,
                                      guint32 width, guint32 height,
                                      WallpiperMonitorGeometry *out) {
  char x11_name[64];
  if (!wallpiper_x11_output_for_size(width, height, x11_name,
                                     sizeof(x11_name))) {
    g_warning("wallpiper-gnome: XRandR has no output currently sized %ux%u",
              width, height);
    return FALSE;
  }

  WallpiperMonitorGeometry monitors[WP_MAX_CAPTURE_CHANNELS];
  char names[WP_MAX_CAPTURE_CHANNELS][64];
  guint count = 0;
  wallpiper_monitor_detect_all_named(state->backend, monitors, &names[0][0],
                                     sizeof(names[0]), WP_MAX_CAPTURE_CHANNELS,
                                     &count);

  for (guint j = 0; j < count; j++) {
    if (g_strcmp0(names[j], x11_name) != 0) {
      continue;
    }
    for (int i = 0; i < WP_MAX_CAPTURE_CHANNELS; i++) {
      if (state->channels[i].active &&
          state->channels[i].monitor.x == monitors[j].x &&
          state->channels[i].monitor.y == monitors[j].y) {
        g_warning("wallpiper-gnome: XRandR output %s (Mutter monitor at "
                  "%d,%d) already claimed by another channel",
                  x11_name, monitors[j].x, monitors[j].y);
        return FALSE;
      }
    }
    *out = monitors[j];
    return TRUE;
  }

  g_warning("wallpiper-gnome: XRandR output %s (matched %ux%u) not found "
            "among Mutter's monitors",
            x11_name, width, height);
  return FALSE;
}

static gboolean find_monitor_for_position(WallpiperPortalState *state,
                                          gint32 x, gint32 y,
                                          WallpiperMonitorGeometry *out) {
  char x11_name[64];
  if (!wallpiper_x11_output_for_position(x, y, x11_name, sizeof(x11_name))) {
    return FALSE;
  }

  WallpiperMonitorGeometry monitors[WP_MAX_CAPTURE_CHANNELS];
  char names[WP_MAX_CAPTURE_CHANNELS][64];
  guint count = 0;
  wallpiper_monitor_detect_all_named(state->backend, monitors, &names[0][0],
                                     sizeof(names[0]), WP_MAX_CAPTURE_CHANNELS,
                                     &count);

  for (guint j = 0; j < count; j++) {
    if (g_strcmp0(names[j], x11_name) != 0) {
      continue;
    }
    for (int i = 0; i < WP_MAX_CAPTURE_CHANNELS; i++) {
      if (state->channels[i].active &&
          state->channels[i].monitor.x == monitors[j].x &&
          state->channels[i].monitor.y == monitors[j].y) {
        g_warning("wallpiper-gnome: XRandR output %s (Mutter monitor at "
                  "%d,%d) already claimed by another channel",
                  x11_name, monitors[j].x, monitors[j].y);
        return FALSE;
      }
    }
    *out = monitors[j];
    return TRUE;
  }

  g_warning("wallpiper-gnome: XRandR output %s (matched position %d,%d) not "
            "found among Mutter's monitors",
            x11_name, x, y);
  return FALSE;
}

static ClutterActor *create_channel_actor(WallpiperPortalState *state,
                                          const WallpiperMonitorGeometry *m) {
  ClutterActor *actor = clutter_actor_new();
  clutter_actor_set_position(actor, m->x, m->y);
  clutter_actor_set_size(actor, m->logical_width, m->logical_height);
  clutter_actor_show(actor);

  ClutterActor *background_actor =
      wallpiper_actor_stacking_dump_children(state->parent, "new-channel");
  if (background_actor) {
    clutter_actor_insert_child_above(state->parent, actor, background_actor);
  } else {
    ClutterActor *current_bottom = clutter_actor_get_first_child(state->parent);
    if (current_bottom)
      clutter_actor_insert_child_above(state->parent, actor, current_bottom);
    else
      clutter_actor_add_child(state->parent, actor);
  }
  return actor;
}

static void display_slot(WallpiperCaptureChannel *ch,
                         WallpiperCaptureSlot *slot) {
  ClutterContent *content =
      clutter_texture_content_new_from_texture(slot->texture, NULL);
  clutter_actor_set_content(ch->display_actor, content);
}

static gboolean recv_capture_message(int sock_fd, char *header_buf,
                                     gsize header_buf_size,
                                     gssize *out_header_len, int *out_fds,
                                     int *out_n_fds) {
  struct iovec iov = {
      .iov_base = header_buf,
      .iov_len = header_buf_size,
  };
  char cmsg_buf[64];
  struct msghdr msg;
  memset(&msg, 0, sizeof(msg));
  msg.msg_iov = &iov;
  msg.msg_iovlen = 1;
  msg.msg_control = cmsg_buf;
  msg.msg_controllen = sizeof(cmsg_buf);

  ssize_t n = recvmsg(sock_fd, &msg, 0);
  if (n < 0)
    return FALSE;

  *out_header_len = n;
  *out_n_fds = 0;

  struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
  if (cmsg && cmsg->cmsg_level == SOL_SOCKET && cmsg->cmsg_type == SCM_RIGHTS) {
    size_t data_len = cmsg->cmsg_len - CMSG_LEN(0);
    int count = (int)(data_len / sizeof(int));
    if (count > MAX_RECV_FDS)
      count = MAX_RECV_FDS;
    int *fds = (int *)CMSG_DATA(cmsg);
    for (int i = 0; i < count; i++)
      out_fds[i] = fds[i];
    *out_n_fds = count;
  }

  return TRUE;
}

static void handle_buf_message(WallpiperPortalState *state, char **parts,
                               guint n_parts, int *fds, int n_fds) {
  if (n_parts < 7) {
    g_warning("wallpiper-gnome: malformed BUF message (%u fields)", n_parts);
    for (int i = 0; i < n_fds; i++)
      close(fds[i]);
    return;
  }

  guint32 wire_slot = (guint32)g_ascii_strtoull(parts[1], NULL, 10);
  guint32 width = (guint32)g_ascii_strtoull(parts[2], NULL, 10);
  guint32 height = (guint32)g_ascii_strtoull(parts[3], NULL, 10);
  guint32 stride = (guint32)g_ascii_strtoull(parts[5], NULL, 10);
  guint64 modifier = g_ascii_strtoull(parts[6], NULL, 10);

  gboolean has_geometry = FALSE;
  gint32 geom_x = 0;
  gint32 geom_y = 0;
  if (n_parts >= 9) {
    has_geometry = TRUE;
    geom_x = (gint32)g_ascii_strtoll(parts[7], NULL, 10);
    geom_y = (gint32)g_ascii_strtoll(parts[8], NULL, 10);
  }

  int dmabuf_fd = n_fds > 0 ? fds[0] : -1;
  int sync_fd = n_fds > 1 ? fds[1] : -1;
  wallpiper_egl_wait_sync_fd(state->egl_display, sync_fd);

  if (dmabuf_fd < 0 || wire_slot >= MAX_CAPTURE_SLOTS) {
    g_warning("wallpiper-gnome: bad BUF message (slot=%u fd=%d)", wire_slot,
              dmabuf_fd);
    if (dmabuf_fd >= 0)
      close(dmabuf_fd);
    return;
  }

  guint32 channel_idx = wire_slot / WP_CAPTURE_SLOT_COUNT;
  guint32 local_idx = wire_slot % WP_CAPTURE_SLOT_COUNT;
  WallpiperCaptureChannel *ch = &state->channels[channel_idx];

  if (!ch->active) {
    WallpiperMonitorGeometry monitor;
    gboolean matched = FALSE;
    if (has_geometry) {
      matched = find_monitor_for_position(state, geom_x, geom_y, &monitor);
    }
    if (!matched) {
      matched = find_monitor_for_size(state, width, height, &monitor);
    }
    if (!matched) {
      g_warning("wallpiper-gnome: no monitor available for new channel %u "
                "(%ux%u), dropping",
                channel_idx, width, height);
      close(dmabuf_fd);
      return;
    }
    ch->monitor = monitor;
    ch->display_actor = create_channel_actor(state, &monitor);
    ch->active = TRUE;
    g_message("wallpiper-gnome: channel %u bound to monitor at (%d,%d) %ux%u "
              "for stream %ux%u%s",
              channel_idx, monitor.x, monitor.y, monitor.width, monitor.height,
              width, height, has_geometry ? " (matched by real window position)" : "");
  }

  g_message("wallpiper-gnome: BUF channel=%u local=%u %ux%u stride=%u "
            "modifier=0x%llx fd=%d",
            channel_idx, local_idx, width, height, stride,
            (unsigned long long)modifier, dmabuf_fd);

  GError *local_error = NULL;
  CoglTexture *texture = wallpiper_egl_import_dmabuf(
      state->cogl_context, state->egl_display, dmabuf_fd, width, height, stride,
      0, modifier, &local_error);
  close(dmabuf_fd);

  if (!texture) {
    g_warning("wallpiper-gnome: failed to import BUF channel=%u local=%u: %s",
              channel_idx, local_idx,
              local_error ? local_error->message : "unknown error");
    g_clear_error(&local_error);
    return;
  }

  WallpiperCaptureSlot *slot = &ch->slots[local_idx];
  if (slot->texture)
    g_object_unref(slot->texture);
  slot->used = TRUE;
  slot->width = width;
  slot->height = height;
  slot->texture = texture;

  display_slot(ch, slot);

  g_message("wallpiper-gnome: displaying channel=%u local=%u", channel_idx,
            local_idx);
}

static void handle_frame_message(WallpiperPortalState *state, char **parts,
                                 guint n_parts, int *fds, int n_fds) {
  int sync_fd = n_fds > 0 ? fds[0] : -1;
  wallpiper_egl_wait_sync_fd(state->egl_display, sync_fd);

  if (n_parts < 2)
    return;

  guint32 wire_slot = (guint32)g_ascii_strtoull(parts[1], NULL, 10);
  if (wire_slot >= MAX_CAPTURE_SLOTS)
    return;

  guint32 channel_idx = wire_slot / WP_CAPTURE_SLOT_COUNT;
  guint32 local_idx = wire_slot % WP_CAPTURE_SLOT_COUNT;
  WallpiperCaptureChannel *ch = &state->channels[channel_idx];
  if (!ch->active || !ch->slots[local_idx].used)
    return;

  display_slot(ch, &ch->slots[local_idx]);
  g_message("wallpiper-gnome: FRAME -> displaying channel=%u local=%u",
            channel_idx, local_idx);
}

static gboolean on_capture_socket_readable(gint fd, GIOCondition condition,
                                           gpointer user_data) {
  WallpiperPortalState *state = user_data;

  if (condition & (G_IO_ERR | G_IO_HUP)) {
    g_warning("wallpiper-gnome: capture socket error/hangup");
    return G_SOURCE_REMOVE;
  }

  char header_buf[256];
  gssize header_len = 0;
  int fds[MAX_RECV_FDS];
  int n_fds = 0;

  if (!recv_capture_message(fd, header_buf, sizeof(header_buf) - 1, &header_len,
                            fds, &n_fds)) {
    g_warning("wallpiper-gnome: recvmsg failed: %s", g_strerror(errno));
    return G_SOURCE_CONTINUE;
  }

  header_buf[header_len] = '\0';

  char **parts = g_strsplit(g_strstrip(header_buf), " ", -1);
  guint n_parts = g_strv_length(parts);

  if (n_parts >= 1 && g_strcmp0(parts[0], "BUF") == 0)
    handle_buf_message(state, parts, n_parts, fds, n_fds);
  else if (n_parts >= 1 && g_strcmp0(parts[0], "FRAME") == 0)
    handle_frame_message(state, parts, n_parts, fds, n_fds);
  else {
    g_message("wallpiper-gnome: unrecognized capture message: %s", header_buf);
    for (int i = 0; i < n_fds; i++)
      close(fds[i]);
  }

  g_strfreev(parts);
  return G_SOURCE_CONTINUE;
}

gboolean wallpiper_capture_listener_start(WallpiperPortalState *state,
                                          GError **error) {
  int capture_fd =
      socket(AF_UNIX, SOCK_DGRAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0);
  if (capture_fd < 0) {
    g_set_error(error, WALLPIPER_ERROR, 0, "capture socket() failed: %s",
                g_strerror(errno));
    return FALSE;
  }

  unlink(WALLPIPER_CAPTURE_SOCKET_PATH);

  struct sockaddr_un addr;
  memset(&addr, 0, sizeof(addr));
  addr.sun_family = AF_UNIX;
  g_strlcpy(addr.sun_path, WALLPIPER_CAPTURE_SOCKET_PATH,
            sizeof(addr.sun_path));
  if (bind(capture_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
    g_set_error(error, WALLPIPER_ERROR, 0, "bind(%s) failed: %s",
                WALLPIPER_CAPTURE_SOCKET_PATH, g_strerror(errno));
    close(capture_fd);
    return FALSE;
  }

  state->capture_socket_fd = capture_fd;
  state->capture_source_id =
      g_unix_fd_add(capture_fd, G_IO_IN, on_capture_socket_readable, state);

  return TRUE;
}

void wallpiper_capture_listener_stop(WallpiperPortalState *state) {
  g_source_remove(state->capture_source_id);
  close(state->capture_socket_fd);
  unlink(WALLPIPER_CAPTURE_SOCKET_PATH);

  for (int i = 0; i < WP_MAX_CAPTURE_CHANNELS; i++) {
    channel_clear(&state->channels[i]);
  }
}
