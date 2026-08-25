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

#include "config.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#include "wallpiper/protocol.h"
#include "wallpiper/steam_paths.h"

static bool fmt_ok(char *out, size_t out_len, const char *fmt, ...) {
  va_list args;
  va_start(args, fmt);
  int n = vsnprintf(out, out_len, fmt, args);
  va_end(args);
  return n > 0 && (size_t)n < out_len;
}

bool wp_renderer_pid_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiper-renderer-pid", temp_dir);
}

bool wp_wrapper_pid_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiper-wrapper-pid", temp_dir);
}

bool wp_tray_icon_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiper-tray-icon", temp_dir);
}

bool wp_tray_click_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiper-tray-click", temp_dir);
}

bool wp_menu_file_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiper-menu-dump", temp_dir);
}

bool wp_menu_command_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiper-menu-command", temp_dir);
}

static bool tray_opts_is(const char *value) {
  const char *v = getenv("WALLPIPER_TRAY_OPTS");
  return v && strcmp(v, value) == 0;
}

bool wp_tray_notray(void) { return tray_opts_is("notray"); }

bool wp_tray_passthrough(void) { return tray_opts_is("passthrough"); }

bool wp_cursor_pos_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiper-cursor", temp_dir);
}

bool wp_wine_fonts_dir(char *out, size_t out_len) {
  char compatdata[768];
  char err[256];
  if (!wp_compatdata_dir(compatdata, sizeof(compatdata), err, sizeof(err))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/pfx/drive_c/windows/Fonts", compatdata);
}

bool wp_font_cache_dir(char *out, size_t out_len) {
  char home[512];
  if (!wp_home_dir(home, sizeof(home))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/.cache/wallpiper/fonts", home);
}

bool wp_preload_path(char *out, size_t out_len) {
  char install_dir[1024];
  if (!wp_install_dir(install_dir, sizeof(install_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/libwallpiper-preload.so", install_dir);
}

bool wp_dwmapi_shim_path(char *out, size_t out_len) {
  char install_dir[1024];
  if (!wp_install_dir(install_dir, sizeof(install_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/dwmapi.dll", install_dir);
}

bool wp_wine_builtin_dwmapi_path(char *out, size_t out_len) {
  char proton[1024];
  char err[256];
  if (!wp_proton_bin(proton, sizeof(proton), err, sizeof(err))) {
    return false;
  }

  char *slash = strrchr(proton, '/');
  if (!slash) {
    return false;
  }
  *slash = '\0';

  char candidate[1024];
  if (!fmt_ok(candidate, sizeof(candidate),
              "%s/files/lib/wine/x86_64-windows/dwmapi.dll", proton)) {
    return false;
  }

  struct stat st;
  if (stat(candidate, &st) != 0 || !S_ISREG(st.st_mode)) {
    return false;
  }
  return fmt_ok(out, out_len, "%s", candidate);
}

bool wp_vk_layer_path(char *out, size_t out_len) {
  return wp_temp_dir(out, out_len);
}
