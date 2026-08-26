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

#include "process.h"

#include <dirent.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

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
