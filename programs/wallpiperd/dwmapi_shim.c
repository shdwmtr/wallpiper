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

#include "dwmapi_shim.h"

#include "config.h"

#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#include "wallpiper/fsutil.h"
#include "wallpiper/steam_paths.h"

static void remove_stale_prefix_override(void) {
  char compatdata[768];
  char err[256];
  if (!wp_compatdata_dir(compatdata, sizeof(compatdata), err, sizeof(err))) {
    return;
  }

  char stale[1024];
  snprintf(stale, sizeof(stale), "%s/pfx/drive_c/windows/system32/dwmapi.dll",
           compatdata);

  struct stat st;
  if (lstat(stale, &st) != 0) {
    return;
  }
  if (S_ISLNK(st.st_mode)) {
    return;
  }

  if (unlink(stale) != 0) {
    printf("failed to remove stale dwmapi shim from prefix system32 (%s)\n",
           stale);
  } else {
    printf("removed stale dwmapi shim from prefix system32 (%s)\n", stale);
  }
}

static bool copy_file(const char *src, const char *dst) {
  int in_fd = open(src, O_RDONLY);
  if (in_fd < 0) {
    return false;
  }
  int out_fd = open(dst, O_WRONLY | O_CREAT | O_TRUNC, 0755);
  if (out_fd < 0) {
    close(in_fd);
    return false;
  }

  char buf[65536];
  bool ok = true;
  ssize_t n;
  while ((n = read(in_fd, buf, sizeof(buf))) > 0) {
    ssize_t off = 0;
    while (off < n) {
      ssize_t written = write(out_fd, buf + off, (size_t)(n - off));
      if (written <= 0) {
        ok = false;
        break;
      }
      off += written;
    }
    if (!ok) {
      break;
    }
  }
  if (n < 0) {
    ok = false;
  }

  close(in_fd);
  close(out_fd);
  return ok;
}

static bool install_shim(const char *src, const char *dst) {
  wp_mkdir_p_parent(dst);

  struct stat st;
  if (lstat(dst, &st) == 0) {
    unlink(dst);
  }

  return copy_file(src, dst);
}

void wp_dwmapi_shim_wire_up(void) {
  remove_stale_prefix_override();

  int arch = wp_we_last_known_arch();
  char shim[1024];
  bool have_shim = false;
  if (arch == 32) {
    have_shim = wp_dwmapi_shim_path32(shim, sizeof(shim));
    struct stat st32;
    if (have_shim && (stat(shim, &st32) != 0 || !S_ISREG(st32.st_mode))) {
      printf("WE last ran as 32-bit but no dwmapi32.dll shim "
             "is installed\n");
      have_shim = false;
    }
  }
  if (!have_shim) {
    have_shim = wp_dwmapi_shim_path(shim, sizeof(shim));
  }
  if (!have_shim) {
    printf("could not resolve dwmapi shim path, skipping "
           "RegisterWaitForSingleObject workaround\n");
    return;
  }
  struct stat st;
  if (stat(shim, &st) != 0 || !S_ISREG(st.st_mode)) {
    printf("dwmapi shim not found at %s, skipping RegisterWaitForSingleObject "
           "workaround\n",
           shim);
    return;
  }

  char orig[1024];
  if (!wp_wine_builtin_dwmapi_path(orig, sizeof(orig))) {
    printf("could not locate Wine's builtin dwmapi.dll, skipping "
           "RegisterWaitForSingleObject workaround\n");
    return;
  }

  char we_exe[1024];
  char err[256];
  if (!wp_we_exe(we_exe, sizeof(we_exe), err, sizeof(err))) {
    printf("could not resolve wallpaper64.exe's directory, skipping "
           "RegisterWaitForSingleObject workaround\n");
    return;
  }
  char *slash = strrchr(we_exe, '/');
  if (!slash) {
    printf("could not resolve wallpaper64.exe's directory, skipping "
           "RegisterWaitForSingleObject workaround\n");
    return;
  }
  *slash = '\0';

  char target[1200];
  snprintf(target, sizeof(target), "%s/dwmapi.dll", we_exe);

  if (!install_shim(shim, target)) {
    printf("failed to install dwmapi shim next to wallpaper64.exe (%s), "
           "skipping workaround\n",
           target);
    return;
  }

  char orig_windows[1030];
  if (!wp_to_windows_path(orig, orig_windows, sizeof(orig_windows))) {
    return;
  }

  setenv("WINEDLLOVERRIDES", "dwmapi=n,b", 1);
  setenv("WALLPIPER_DWMAPI_ORIG", orig_windows, 1);
}
