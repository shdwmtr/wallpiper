#include "log.h"
#include <fcntl.h>
#include <pthread.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#define CURSOR_POLL_INTERVAL_US 16000

#define TARGET_PROCESS_WAIT_ATTEMPTS 50
#define TARGET_PROCESS_WAIT_INTERVAL_US 100000

static const char *const target_process_names[] = {"wallpaper64.exe"};
static const char *const target_cmdline_markers[] = {"webwallpaper64.exe"};

static bool ctl_socket_path(char *out, size_t out_len) {
  const char *path = getenv("WALLPIPER_PORTAL_CTL_SOCKET");
  if (!path || path[0] == '\0') {
    return false;
  }
  int n = snprintf(out, out_len, "%s", path);
  return n > 0 && (size_t)n < out_len;
}

static bool query_cursor_pos(int *x, int *y) {
  char path[256];
  if (!ctl_socket_path(path, sizeof(path))) {
    return false;
  }
  size_t path_len = strlen(path);
  if (path_len >= sizeof(((struct sockaddr_un *)0)->sun_path)) {
    return false;
  }

  int sock = socket(AF_UNIX, SOCK_STREAM, 0);
  if (sock < 0) {
    return false;
  }

  struct sockaddr_un addr;
  memset(&addr, 0, sizeof(addr));
  addr.sun_family = AF_UNIX;
  memcpy(addr.sun_path, path, path_len);

  if (connect(sock, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
    close(sock);
    return false;
  }

  static const char cmd[] = "CURSOR_POS\n";
  if (write(sock, cmd, sizeof(cmd) - 1) != (ssize_t)(sizeof(cmd) - 1)) {
    close(sock);
    return false;
  }

  char buf[64];
  size_t total = 0;
  while (total < sizeof(buf) - 1) {
    ssize_t n = read(sock, buf + total, sizeof(buf) - 1 - total);
    if (n <= 0) {
      break;
    }
    total += (size_t)n;
  }
  close(sock);
  buf[total] = '\0';

  return sscanf(buf, "CURSOR_POS %d %d", x, y) == 2;
}

static int open_output_file(void) {
  const char *path = getenv("WALLPIPER_CURSOR_POS_FILE");
  if (!path || path[0] == '\0') {
    return -1;
  }
  return open(path, O_WRONLY | O_CREAT, 0644);
}

static bool read_comm(char *out, size_t out_len) {
  FILE *f = fopen("/proc/self/comm", "r");
  if (!f) {
    return false;
  }
  size_t n = fread(out, 1, out_len - 1, f);
  fclose(f);
  while (n > 0 && (out[n - 1] == '\n' || out[n - 1] == '\r')) {
    n--;
  }
  out[n] = '\0';
  return true;
}

bool interpose_is_target_process(void) {
  static int cached = -1;
  int c = __atomic_load_n(&cached, __ATOMIC_RELAXED);
  if (c == 1) {
    return true;
  }

  bool result = false;

  char comm[256];
  if (read_comm(comm, sizeof(comm))) {
    for (size_t i = 0;
         i < sizeof(target_process_names) / sizeof(target_process_names[0]);
         i++) {
      if (strcmp(comm, target_process_names[i]) == 0) {
        result = true;
        break;
      }
    }
  }

  if (!result) {
    FILE *cf = fopen("/proc/self/cmdline", "r");
    if (cf) {
      char buf[4096];
      size_t n = fread(buf, 1, sizeof(buf), cf);
      fclose(cf);
      for (size_t i = 0; i < sizeof(target_cmdline_markers) /
                                 sizeof(target_cmdline_markers[0]);
           i++) {
        size_t marker_len = strlen(target_cmdline_markers[i]);
        if (marker_len > 0 && n >= marker_len &&
            memmem(buf, n, target_cmdline_markers[i], marker_len) != NULL) {
          result = true;
          break;
        }
      }
    }
  }

  if (result) {
    __atomic_store_n(&cached, 1, __ATOMIC_RELAXED);
  }
  return result;
}

static void *cursor_forward_thread(void *arg) {
  (void)arg;

  int attempt = 0;
  while (!interpose_is_target_process()) {
    attempt++;
    if (attempt >= TARGET_PROCESS_WAIT_ATTEMPTS) {
      return NULL;
    }
    usleep(TARGET_PROCESS_WAIT_INTERVAL_US);
  }

  int fd = open_output_file();
  if (fd < 0) {
    wp_log("cursor forward: WALLPIPER_CURSOR_POS_FILE not set or unopenable, "
           "disabled");
    return NULL;
  }

  for (;;) {
    int x, y;
    if (query_cursor_pos(&x, &y)) {
      int32_t payload[2] = {(int32_t)x, (int32_t)y};
      if (pwrite(fd, payload, sizeof(payload), 0) != (ssize_t)sizeof(payload)) {
        wp_log("cursor forward: pwrite failed");
      }
    }
    usleep(CURSOR_POLL_INTERVAL_US);
  }

  return NULL;
}

__attribute__((constructor)) static void wp_start_cursor_forward(void) {
  pthread_t thread;
  if (pthread_create(&thread, NULL, cursor_forward_thread, NULL) != 0) {
    wp_log("cursor forward: pthread_create failed");
    return;
  }
  pthread_detach(thread);
}
