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

#include "wallpiper/protocol.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

static bool env_path(const char *name, const char **out) {
  const char *v = getenv(name);
  if (v && v[0] != '\0') {
    *out = v;
    return true;
  }
  return false;
}

static bool fmt_ok(char *out, size_t out_len, const char *fmt, ...) {
  va_list args;
  va_start(args, fmt);
  int n = vsnprintf(out, out_len, fmt, args);
  va_end(args);
  return n > 0 && (size_t)n < out_len;
}

bool wp_ctl_socket_path(const char *portal_name, char *out, size_t out_len) {
  if (!portal_name || portal_name[0] == '\0') {
    return false;
  }
  return fmt_ok(out, out_len, "/tmp/wallpiper-portal-%s-ctl.sock", portal_name);
}

bool wp_temp_dir(char *out, size_t out_len) {
  const char *v;
  if (env_path("WALLPIPER_TEMP_DIR", &v)) {
    return fmt_ok(out, out_len, "%s", v);
  }
  return fmt_ok(out, out_len, "/tmp/wallpiper");
}

bool wp_daemon_ctl_socket_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiperd-ctl.sock", temp_dir);
}

bool wp_runtime_dir(char *out, size_t out_len) {
  const char *v;
  if (env_path("WALLPIPER_RUNTIME_DIR", &v)) {
    return fmt_ok(out, out_len, "%s", v);
  }

  const char *state_home;
  if (env_path("XDG_STATE_HOME", &state_home)) {
    return fmt_ok(out, out_len, "%s/wallpiper", state_home);
  }

  const char *home;
  if (!env_path("HOME", &home)) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/.local/state/wallpiper", home);
}
