#include "process.h"

#include <dirent.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "wallpiper/steam_paths.h"

static bool pid_alive(int pid) {
  char path[64];
  snprintf(path, sizeof(path), "/proc/%d", pid);
  return access(path, F_OK) == 0;
}

void wp_kill_pids_gracefully(const int *pids, size_t count) {
  for (size_t i = 0; i < count; i++) {
    int res = kill(pids[i], SIGTERM);
    printf("SIGTERM pid=%d -> %s\n", pids[i], res == 0 ? "ok" : "failed");
  }

  for (int attempt = 0; attempt < 30; attempt++) {
    bool any_alive = false;
    for (size_t i = 0; i < count; i++) {
      if (pid_alive(pids[i])) {
        any_alive = true;
        break;
      }
    }
    if (!any_alive) {
      return;
    }
    struct timespec ts = {.tv_sec = 0, .tv_nsec = 100000000L};
    nanosleep(&ts, NULL);
  }

  for (size_t i = 0; i < count; i++) {
    if (pid_alive(pids[i])) {
      int res = kill(pids[i], SIGKILL);
      printf("pid=%d still alive after SIGTERM grace period, SIGKILL -> %s\n",
             pids[i], res == 0 ? "ok" : "failed");
    }
  }
}

static bool read_comm(int pid, char *out, size_t out_len) {
  char path[64];
  snprintf(path, sizeof(path), "/proc/%d/comm", pid);
  FILE *f = fopen(path, "r");
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

static bool read_cmdline(int pid, char *out, size_t out_len) {
  char path[64];
  snprintf(path, sizeof(path), "/proc/%d/cmdline", pid);
  FILE *f = fopen(path, "r");
  if (!f) {
    return false;
  }
  size_t n = fread(out, 1, out_len - 1, f);
  fclose(f);
  out[n] = '\0';
  return true;
}

static void scan_proc_pids(wp_pid_list_t *out,
                           bool (*matches)(int pid, const void *ctx),
                           const void *ctx) {
  out->count = 0;

  DIR *dir = opendir("/proc");
  if (!dir) {
    return;
  }

  struct dirent *entry;
  while ((entry = readdir(dir)) != NULL) {
    char *end = NULL;
    long pid = strtol(entry->d_name, &end, 10);
    if (end == entry->d_name || *end != '\0' || pid <= 0) {
      continue;
    }
    if (matches((int)pid, ctx)) {
      if (out->count < sizeof(out->pids) / sizeof(out->pids[0])) {
        out->pids[out->count++] = (int)pid;
      }
    }
  }
  closedir(dir);
}

static bool comm_is_renderer(int pid, const void *ctx) {
  (void)ctx;
  char comm[256];
  return read_comm(pid, comm, sizeof(comm)) &&
         strcmp(comm, "wallpaper64.exe") == 0;
}

void wp_find_renderer_pids(wp_pid_list_t *out) {
  scan_proc_pids(out, comm_is_renderer, NULL);
}

int wp_find_renderer_pid(void) {
  wp_pid_list_t list;
  wp_find_renderer_pids(&list);
  return list.count > 0 ? list.pids[0] : -1;
}

static bool pid_has_env_var(int pid, const char *key, const char *value) {
  char path[64];
  snprintf(path, sizeof(path), "/proc/%d/environ", pid);
  FILE *f = fopen(path, "rb");
  if (!f) {
    return false;
  }

  char marker[1024];
  int marker_len = snprintf(marker, sizeof(marker), "%s=%s", key, value);
  if (marker_len <= 0 || (size_t)marker_len >= sizeof(marker)) {
    fclose(f);
    return false;
  }

  char buf[16384];
  size_t total = fread(buf, 1, sizeof(buf) - 1, f);
  fclose(f);
  buf[total] = '\0';

  size_t offset = 0;
  while (offset < total) {
    const char *entry = buf + offset;
    size_t entry_len = strlen(entry);
    if (entry_len == (size_t)marker_len &&
        memcmp(entry, marker, entry_len) == 0) {
      return true;
    }
    offset += entry_len + 1;
  }
  return false;
}

typedef struct {
  char compatdata[768];
} location_ctx_t;

static bool matches_location(int pid, const void *ctx) {
  const location_ctx_t *lctx = ctx;
  return pid_has_env_var(pid, "STEAM_COMPAT_DATA_PATH", lctx->compatdata);
}

static bool build_location_ctx(const char *location, location_ctx_t *ctx) {
  char err[256];
  return wp_compatdata_for(location, ctx->compatdata, sizeof(ctx->compatdata),
                           err, sizeof(err));
}

void wp_find_renderer_pids_for_location(const char *location,
                                        wp_pid_list_t *out) {
  out->count = 0;
  location_ctx_t ctx;
  if (!build_location_ctx(location, &ctx)) {
    return;
  }

  wp_pid_list_t candidates;
  scan_proc_pids(&candidates, matches_location, &ctx);
  for (size_t i = 0; i < candidates.count; i++) {
    if (comm_is_renderer(candidates.pids[i], NULL)) {
      out->pids[out->count++] = candidates.pids[i];
    }
  }
}

static bool comm_is_python3(int pid, const void *ctx) {
  (void)ctx;
  char comm[256];
  return read_comm(pid, comm, sizeof(comm)) && strcmp(comm, "python3") == 0;
}

void wp_find_proton_wrapper_pids_for_location(const char *location,
                                              wp_pid_list_t *out) {
  out->count = 0;
  location_ctx_t ctx;
  if (!build_location_ctx(location, &ctx)) {
    return;
  }

  wp_pid_list_t candidates;
  scan_proc_pids(&candidates, matches_location, &ctx);
  for (size_t i = 0; i < candidates.count; i++) {
    if (comm_is_python3(candidates.pids[i], NULL)) {
      out->pids[out->count++] = candidates.pids[i];
    }
  }
}

static bool cmdline_contains(int pid, const void *ctx) {
  const char *needle = ctx;
  char cmdline[8192];
  if (!read_cmdline(pid, cmdline, sizeof(cmdline))) {
    return false;
  }
  return memmem(cmdline, strlen(cmdline) + 1, needle, strlen(needle)) != NULL;
}

void wp_find_picker_pids(wp_pid_list_t *out) {
  scan_proc_pids(out, cmdline_contains, "wallpaperui.exe");
}

void wp_find_webwallpaper_pids(wp_pid_list_t *out) {
  scan_proc_pids(out, cmdline_contains, "webwallpaper64.exe");
}
