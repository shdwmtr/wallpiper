#include "wallpiper/misc_ipc.h"
#include "wallpiper/protocol.h"

#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/un.h>
#include <unistd.h>

static int connect_with_timeouts(const char *path, long read_secs,
                                 long write_secs, char *err_out,
                                 size_t err_out_len) {
  size_t path_len = strlen(path);
  if (path_len >= sizeof(((struct sockaddr_un *)0)->sun_path)) {
    snprintf(err_out, err_out_len, "socket path too long: %s", path);
    return -1;
  }

  int sock = socket(AF_UNIX, SOCK_STREAM, 0);
  if (sock < 0) {
    snprintf(err_out, err_out_len, "could not create socket");
    return -1;
  }

  struct sockaddr_un addr;
  memset(&addr, 0, sizeof(addr));
  addr.sun_family = AF_UNIX;
  memcpy(addr.sun_path, path, path_len);

  if (connect(sock, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
    snprintf(err_out, err_out_len, "not available (no socket at %s)", path);
    close(sock);
    return -1;
  }

  struct timeval read_tv = {.tv_sec = read_secs, .tv_usec = 0};
  struct timeval write_tv = {.tv_sec = write_secs, .tv_usec = 0};
  setsockopt(sock, SOL_SOCKET, SO_RCVTIMEO, &read_tv, sizeof(read_tv));
  setsockopt(sock, SOL_SOCKET, SO_SNDTIMEO, &write_tv, sizeof(write_tv));

  return sock;
}

static bool read_one_line(int sock, char *out, size_t out_len) {
  size_t total = 0;
  while (total + 1 < out_len) {
    char c;
    ssize_t n = read(sock, &c, 1);
    if (n <= 0) {
      break;
    }
    out[total++] = c;
    if (c == '\n') {
      break;
    }
  }
  out[total] = '\0';
  while (total > 0 && (out[total - 1] == '\n' || out[total - 1] == '\r')) {
    out[--total] = '\0';
  }
  return total > 0;
}

bool wp_send_windowbrowser_trigger(const char *const *keysyms,
                                   size_t keysym_count, char *err_out,
                                   size_t err_out_len) {
  char path[512];
  if (!wp_windowbrowser_socket_path(path, sizeof(path))) {
    snprintf(err_out, err_out_len,
             "could not resolve window browser socket path");
    return false;
  }

  int sock = connect_with_timeouts(path, 3, 2, err_out, err_out_len);
  if (sock < 0) {
    return false;
  }

  char request[1024];
  size_t offset = (size_t)snprintf(request, sizeof(request), "TRIGGER");
  for (size_t i = 0; i < keysym_count; i++) {
    int n =
        snprintf(request + offset, sizeof(request) - offset, " %s", keysyms[i]);
    if (n <= 0 || offset + (size_t)n >= sizeof(request)) {
      snprintf(err_out, err_out_len, "trigger too long");
      close(sock);
      return false;
    }
    offset += (size_t)n;
  }
  request[offset++] = '\n';

  if (write(sock, request, offset) != (ssize_t)offset) {
    snprintf(err_out, err_out_len, "failed to write trigger");
    close(sock);
    return false;
  }

  char line[512];
  bool got_line = read_one_line(sock, line, sizeof(line));
  close(sock);
  if (!got_line) {
    snprintf(err_out, err_out_len, "no response from window browser listener");
    return false;
  }

  if (strcmp(line, "OK") == 0) {
    return true;
  }
  if (strncmp(line, "ERR", 3) == 0) {
    const char *rest = line + 3;
    while (*rest == ' ' || *rest == '\t') {
      rest++;
    }
    snprintf(err_out, err_out_len, "%s", rest);
    return false;
  }
  snprintf(err_out, err_out_len,
           "malformed response from window browser listener: %s", line);
  return false;
}

bool wp_send_inject_select(const char *unix_path, const char *location,
                           char *reply_out, size_t reply_out_len, char *err_out,
                           size_t err_out_len) {
  char path[512];
  if (!wp_inject_socket_path(path, sizeof(path))) {
    snprintf(err_out, err_out_len, "could not resolve live-inject socket path");
    return false;
  }

  int sock = connect_with_timeouts(path, 5, 2, err_out, err_out_len);
  if (sock < 0) {
    return false;
  }

  char request[1024];
  int n =
      snprintf(request, sizeof(request), "SELECT %s|%s\n", unix_path, location);
  if (n <= 0 || (size_t)n >= sizeof(request)) {
    snprintf(err_out, err_out_len, "select request too long");
    close(sock);
    return false;
  }

  if (write(sock, request, (size_t)n) != n) {
    snprintf(err_out, err_out_len, "failed to write select request");
    close(sock);
    return false;
  }

  char line[512];
  bool got_line = read_one_line(sock, line, sizeof(line));
  close(sock);
  if (!got_line) {
    snprintf(err_out, err_out_len, "no response from live-inject listener");
    return false;
  }

  if (strncmp(line, "OK", 2) == 0) {
    if (reply_out && reply_out_len > 0) {
      snprintf(reply_out, reply_out_len, "%s", line);
    }
    return true;
  }
  if (strncmp(line, "ERR", 3) == 0) {
    const char *rest = line + 3;
    while (*rest == ' ' || *rest == '\t') {
      rest++;
    }
    snprintf(err_out, err_out_len, "%s", rest);
    return false;
  }
  snprintf(err_out, err_out_len,
           "malformed response from live-inject listener: %s", line);
  return false;
}
