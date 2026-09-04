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

#include "logging.h"
#include "process.h"

#include <pthread.h>
#include <stdio.h>
#include <string.h>

static char g_detected_comm[256] = "";

static bool detect_target_process(void) {
  FILE *f = fopen("/proc/self/comm", "r");
  if (f) {
    char comm[256];
    size_t n = fread(comm, 1, sizeof(comm) - 1, f);
    fclose(f);
    while (n > 0 && (comm[n - 1] == '\n' || comm[n - 1] == '\r')) {
      n--;
    }
    comm[n] = '\0';
    snprintf(g_detected_comm, sizeof(g_detected_comm), "%s", comm);
    if (strcmp(comm, "wallpaper64.exe") == 0 ||
        strcmp(comm, "wallpaper32.exe") == 0) {
      return true;
    }
  }

  f = fopen("/proc/self/cmdline", "r");
  if (f) {
    char cmdline[8192];
    size_t n = fread(cmdline, 1, sizeof(cmdline) - 1, f);
    fclose(f);
    cmdline[n] = '\0';
    if (memmem(cmdline, n, "webwallpaper64.exe",
               strlen("webwallpaper64.exe")) != NULL ||
        memmem(cmdline, n, "webwallpaper32.exe",
               strlen("webwallpaper32.exe")) != NULL) {
      return true;
    }
  }

  return false;
}

static pthread_once_t g_once = PTHREAD_ONCE_INIT;
static bool g_is_target = false;

static void init_once(void) {
  g_is_target = detect_target_process();
  if (!g_is_target) {
    WP_LOG("capture layer loaded into unrecognized process %s%s%s, frame "
           "capture disabled for this process",
           g_detected_comm[0] ? "'" : "(comm unavailable)",
           g_detected_comm[0] ? g_detected_comm : "",
           g_detected_comm[0] ? "'" : "");
  }
}

bool wp_capture_is_target_process(void) {
  pthread_once(&g_once, init_once);
  return g_is_target;
}
