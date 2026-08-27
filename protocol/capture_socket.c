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

#include "wallpiper/capture_socket.h"
#include "wallpiper/log.h"
#include "wallpiper/protocol.h"

#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

static int bind_unix_dgram(const char *path) {
  unlink(path);

  int sock = socket(AF_UNIX, SOCK_DGRAM | SOCK_CLOEXEC, 0);
  if (sock < 0) {
    return -1;
  }

  struct sockaddr_un addr;
  memset(&addr, 0, sizeof(addr));
  addr.sun_family = AF_UNIX;
  size_t path_len = strlen(path);
  if (path_len >= sizeof(addr.sun_path)) {
    close(sock);
    return -1;
  }
  memcpy(addr.sun_path, path, path_len);

  if (bind(sock, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
    close(sock);
    return -1;
  }
  return sock;
}

int wp_bind_capture_socket(void) {
  int sock = bind_unix_dgram(WALLPIPER_CAPTURE_SOCKET_PATH);
  if (sock < 0) {
    return -1;
  }

  int flags = fcntl(sock, F_GETFL, 0);
  if (flags >= 0) {
    fcntl(sock, F_SETFL, flags | O_NONBLOCK);
  }

  wp_log("capture socket listening on %s", WALLPIPER_CAPTURE_SOCKET_PATH);
  return sock;
}

static ssize_t recv_msg(int sock_fd, char *header_buf, size_t header_buf_len,
                        int *fds, int max_fds, int *nfds) {
  char cmsg_buf[64];

  struct iovec iov = {
      .iov_base = header_buf,
      .iov_len = header_buf_len,
  };

  struct msghdr msg;
  memset(&msg, 0, sizeof(msg));
  msg.msg_iov = &iov;
  msg.msg_iovlen = 1;
  msg.msg_control = cmsg_buf;
  msg.msg_controllen = sizeof(cmsg_buf);

  ssize_t n = recvmsg(sock_fd, &msg, 0);
  if (n < 0) {
    *nfds = 0;
    return n;
  }

  *nfds = 0;
  struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
  if (cmsg && cmsg->cmsg_level == SOL_SOCKET && cmsg->cmsg_type == SCM_RIGHTS) {
    size_t data_len = cmsg->cmsg_len - CMSG_LEN(0);
    size_t count = data_len / sizeof(int);
    if ((int)count > max_fds) {
      count = (size_t)max_fds;
    }
    const int *data = (const int *)CMSG_DATA(cmsg);
    for (size_t i = 0; i < count; i++) {
      fds[i] = data[i];
    }
    *nfds = (int)count;
  }

  return n;
}

static void close_fds(const int *fds, int nfds) {
  for (int i = 0; i < nfds; i++) {
    close(fds[i]);
  }
}

static bool parse_event(const char *header, const int *fds, int nfds,
                        wp_capture_event_t *out) {
  char buf[256];
  snprintf(buf, sizeof(buf), "%s", header);

  char *saveptr = NULL;
  char *tok = strtok_r(buf, " \t", &saveptr);
  if (!tok) {
    return false;
  }

  memset(out, 0, sizeof(*out));

  if (strcmp(tok, "BUF") == 0) {
    char *slot_s = strtok_r(NULL, " \t", &saveptr);
    char *width_s = strtok_r(NULL, " \t", &saveptr);
    char *height_s = strtok_r(NULL, " \t", &saveptr);
    char *format_s = strtok_r(NULL, " \t", &saveptr);
    char *stride_s = strtok_r(NULL, " \t", &saveptr);
    char *modifier_s = strtok_r(NULL, " \t", &saveptr);
    char *geom_x_s = strtok_r(NULL, " \t", &saveptr);
    char *geom_y_s = strtok_r(NULL, " \t", &saveptr);
    if (!slot_s || !width_s || !height_s || !format_s || !stride_s ||
        !modifier_s || nfds < 1) {
      return false;
    }
    out->tag = WP_CAPTURE_EVENT_BUF;
    out->slot = (uint32_t)strtoul(slot_s, NULL, 10);
    out->width = (uint32_t)strtoul(width_s, NULL, 10);
    out->height = (uint32_t)strtoul(height_s, NULL, 10);
    out->stride = (uint32_t)strtoul(stride_s, NULL, 10);
    out->modifier = (uint64_t)strtoull(modifier_s, NULL, 10);
    if (geom_x_s && geom_y_s) {
      out->has_geometry = true;
      out->geom_x = (int32_t)strtol(geom_x_s, NULL, 10);
      out->geom_y = (int32_t)strtol(geom_y_s, NULL, 10);
    }
    out->fds[0] = fds[0];
    out->nfds = 1;
    if (nfds > 1) {
      out->fds[1] = fds[1];
      out->nfds = 2;
    }
    return true;
  }

  if (strcmp(tok, "FRAME") == 0) {
    char *slot_s = strtok_r(NULL, " \t", &saveptr);
    if (!slot_s) {
      return false;
    }
    out->tag = WP_CAPTURE_EVENT_FRAME;
    out->slot = (uint32_t)strtoul(slot_s, NULL, 10);
    if (nfds > 0) {
      out->fds[0] = fds[0];
      out->nfds = 1;
    }
    return true;
  }

  if (strcmp(tok, "SHM") == 0) {
    char *width_s = strtok_r(NULL, " \t", &saveptr);
    char *height_s = strtok_r(NULL, " \t", &saveptr);
    char *stride_s = strtok_r(NULL, " \t", &saveptr);
    if (!width_s || !height_s || !stride_s || nfds < 1) {
      return false;
    }
    out->tag = WP_CAPTURE_EVENT_SHM;
    out->width = (uint32_t)strtoul(width_s, NULL, 10);
    out->height = (uint32_t)strtoul(height_s, NULL, 10);
    out->stride = (uint32_t)strtoul(stride_s, NULL, 10);
    out->fds[0] = fds[0];
    out->nfds = 1;
    return true;
  }

  return false;
}

bool wp_recv_capture_event(int sock_fd, wp_capture_event_t *out) {
  char header_buf[256];
  int fds[WP_CAPTURE_MAX_FDS];
  int nfds = 0;

  ssize_t n = recv_msg(sock_fd, header_buf, sizeof(header_buf), fds,
                       WP_CAPTURE_MAX_FDS, &nfds);
  if (n < 0) {
    return false;
  }

  size_t len = (size_t)n;
  while (len > 0 &&
         (header_buf[len - 1] == '\n' || header_buf[len - 1] == '\r' ||
          header_buf[len - 1] == ' ')) {
    len--;
  }
  header_buf[len] = '\0';

  if (!parse_event(header_buf, fds, nfds, out)) {
    wp_log("capture socket: unrecognized or malformed message: %s", header_buf);
    close_fds(fds, nfds);
    return false;
  }

  return true;
}

struct wp_capture_link {
  int sock;
};

wp_capture_link_t *wp_capture_link_create(void) {
  wp_capture_link_t *link = calloc(1, sizeof(wp_capture_link_t));
  if (!link) {
    return NULL;
  }
  link->sock = -1;
  return link;
}

void wp_capture_link_destroy(wp_capture_link_t *link) {
  if (!link) {
    return;
  }
  if (link->sock >= 0) {
    close(link->sock);
  }
  free(link);
}

static bool ensure_connected(wp_capture_link_t *link) {
  if (link->sock >= 0) {
    return true;
  }

  int sock = socket(AF_UNIX, SOCK_DGRAM | SOCK_CLOEXEC, 0);
  if (sock < 0) {
    return false;
  }

  struct sockaddr_un addr;
  memset(&addr, 0, sizeof(addr));
  addr.sun_family = AF_UNIX;
  size_t path_len = strlen(WALLPIPER_CAPTURE_SOCKET_PATH);
  memcpy(addr.sun_path, WALLPIPER_CAPTURE_SOCKET_PATH, path_len);

  if (connect(sock, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
    close(sock);
    return false;
  }

  link->sock = sock;
  return true;
}

static void reset_link(wp_capture_link_t *link) {
  if (link->sock >= 0) {
    close(link->sock);
    link->sock = -1;
  }
}

static bool send_with_fds(wp_capture_link_t *link, const char *header,
                          const int *fds, int nfds) {
  if (!ensure_connected(link)) {
    return false;
  }

  struct iovec iov = {
      .iov_base = (void *)header,
      .iov_len = strlen(header),
  };

  struct msghdr msg;
  memset(&msg, 0, sizeof(msg));
  msg.msg_iov = &iov;
  msg.msg_iovlen = 1;

  char cmsg_buf[CMSG_SPACE(sizeof(int) * WP_CAPTURE_MAX_FDS)];
  if (nfds > 0) {
    msg.msg_control = cmsg_buf;
    msg.msg_controllen = CMSG_SPACE(sizeof(int) * (size_t)nfds);
    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;
    cmsg->cmsg_len = CMSG_LEN(sizeof(int) * (size_t)nfds);
    memcpy(CMSG_DATA(cmsg), fds, sizeof(int) * (size_t)nfds);
  }

  ssize_t sent = sendmsg(link->sock, &msg, 0);
  bool ok = sent >= 0;
  if (!ok) {
    reset_link(link);
  }
  return ok;
}

bool wp_capture_link_send_buf(wp_capture_link_t *link, uint32_t slot,
                              uint32_t width, uint32_t height,
                              uint32_t format_raw, uint32_t stride,
                              uint64_t modifier, bool has_geometry,
                              int32_t geom_x, int32_t geom_y, int image_fd,
                              int sync_fd) {
  char header[160];
  if (has_geometry) {
    snprintf(header, sizeof(header), "BUF %u %u %u %u %u %llu %d %d\n", slot,
             width, height, format_raw, stride,
             (unsigned long long)modifier, geom_x, geom_y);
  } else {
    snprintf(header, sizeof(header), "BUF %u %u %u %u %u %llu\n", slot, width,
             height, format_raw, stride, (unsigned long long)modifier);
  }

  int fds[WP_CAPTURE_MAX_FDS];
  int nfds = 0;
  if (image_fd >= 0) {
    fds[nfds++] = image_fd;
  }
  if (sync_fd >= 0) {
    fds[nfds++] = sync_fd;
  }

  bool ok = send_with_fds(link, header, fds, nfds);

  if (image_fd >= 0) {
    close(image_fd);
  }
  if (sync_fd >= 0) {
    close(sync_fd);
  }
  return ok;
}

bool wp_capture_link_send_frame(wp_capture_link_t *link, uint32_t slot,
                                int sync_fd) {
  char header[64];
  snprintf(header, sizeof(header), "FRAME %u\n", slot);

  int fds[1];
  int nfds = 0;
  if (sync_fd >= 0) {
    fds[nfds++] = sync_fd;
  }

  bool ok = send_with_fds(link, header, fds, nfds);

  if (sync_fd >= 0) {
    close(sync_fd);
  }
  return ok;
}
