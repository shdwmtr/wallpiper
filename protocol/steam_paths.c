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

#include "wallpiper/steam_paths.h"
#include "wallpiper/protocol.h"

#include <ctype.h>
#include <dirent.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/stat.h>
#include <unistd.h>

static bool fmt_ok(char *out, size_t out_len, const char *fmt, ...) {
  va_list args;
  va_start(args, fmt);
  int n = vsnprintf(out, out_len, fmt, args);
  va_end(args);
  return n > 0 && (size_t)n < out_len;
}

static bool is_dir(const char *path) {
  struct stat st;
  return stat(path, &st) == 0 && S_ISDIR(st.st_mode);
}

static bool is_regular_file(const char *path) {
  struct stat st;
  return stat(path, &st) == 0 && S_ISREG(st.st_mode);
}

bool wp_home_dir(char *out, size_t out_len) {
  const char *home = getenv("HOME");
  if (!home || home[0] == '\0') {
    return false;
  }
  return fmt_ok(out, out_len, "%s", home);
}

bool wp_install_dir(char *out, size_t out_len) {
  char exe[1024];
  ssize_t n = readlink("/proc/self/exe", exe, sizeof(exe) - 1);
  if (n <= 0) {
    return false;
  }
  exe[n] = '\0';

  char *slash = strrchr(exe, '/');
  if (!slash) {
    return false;
  }
  *slash = '\0';
  return fmt_ok(out, out_len, "%s", exe);
}

bool wp_steam_root(char *out, size_t out_len, char *err_out,
                   size_t err_out_len) {
  const char *override = getenv("WALLPIPER_STEAM_ROOT");
  if (override && override[0] != '\0') {
    return fmt_ok(out, out_len, "%s", override);
  }

  char home[512];
  if (!wp_home_dir(home, sizeof(home))) {
    snprintf(err_out, err_out_len, "HOME is not set");
    return false;
  }

  char candidates[4][768];
  snprintf(candidates[0], sizeof(candidates[0]), "%s/.local/share/Steam", home);
  snprintf(candidates[1], sizeof(candidates[1]), "%s/.steam/steam", home);
  snprintf(candidates[2], sizeof(candidates[2]), "%s/.steam/root", home);
  snprintf(candidates[3], sizeof(candidates[3]),
           "%s/.var/app/com.valvesoftware.Steam/.local/share/Steam", home);

  for (int i = 0; i < 4; i++) {
    if (is_dir(candidates[i])) {
      return fmt_ok(out, out_len, "%s", candidates[i]);
    }
  }

  snprintf(err_out, err_out_len,
           "could not find a Steam install. set WALLPIPER_STEAM_ROOT to its "
           "path (checked %s, %s, %s, %s)",
           candidates[0], candidates[1], candidates[2], candidates[3]);
  return false;
}

bool wp_compatdata_dir(char *out, size_t out_len, char *err_out,
                       size_t err_out_len) {
  char steam_root[768];
  if (!wp_steam_root(steam_root, sizeof(steam_root), err_out, err_out_len)) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/steamapps/compatdata/%s", steam_root,
                WALLPAPER_ENGINE_APP_ID);
}

bool wp_workshop_content_dir(char *out, size_t out_len, char *err_out,
                             size_t err_out_len) {
  char steam_root[768];
  if (!wp_steam_root(steam_root, sizeof(steam_root), err_out, err_out_len)) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/steamapps/workshop/content/%s", steam_root,
                WALLPAPER_ENGINE_APP_ID);
}

bool wp_we_config_path(char *out, size_t out_len, char *err_out,
                       size_t err_out_len) {
  char steam_root[768];
  if (!wp_steam_root(steam_root, sizeof(steam_root), err_out, err_out_len)) {
    return false;
  }
  return fmt_ok(out, out_len,
                "%s/steamapps/common/wallpaper_engine/config.json", steam_root);
}

bool wp_we_exe(char *out, size_t out_len, char *err_out, size_t err_out_len) {
  const char *override = getenv("WALLPIPER_WE_EXE");
  if (override && override[0] != '\0') {
    return fmt_ok(out, out_len, "%s", override);
  }

  char steam_root[768];
  if (!wp_steam_root(steam_root, sizeof(steam_root), err_out, err_out_len)) {
    return false;
  }

  char path[1024];
  if (!fmt_ok(path, sizeof(path),
              "%s/steamapps/common/wallpaper_engine/wallpaper64.exe",
              steam_root)) {
    snprintf(err_out, err_out_len, "steam root path too long");
    return false;
  }
  if (!is_regular_file(path)) {
    snprintf(err_out, err_out_len,
             "Wallpaper Engine executable not found at %s. set "
             "WALLPIPER_WE_EXE to its path",
             path);
    return false;
  }
  return fmt_ok(out, out_len, "%s", path);
}

static bool scan_proton_candidates(const char *dir, char candidates[][1024],
                                   size_t max_candidates, size_t *count) {
  DIR *d = opendir(dir);
  if (!d) {
    return true;
  }

  struct dirent *entry;
  while ((entry = readdir(d)) != NULL) {
    if (entry->d_name[0] == '.') {
      continue;
    }
    if (*count >= max_candidates) {
      break;
    }

    char candidate[1024];
    int n = snprintf(candidate, sizeof(candidate), "%s/%s/proton", dir,
                     entry->d_name);
    if (n <= 0 || (size_t)n >= sizeof(candidate)) {
      continue;
    }
    if (is_regular_file(candidate)) {
      snprintf(candidates[*count], sizeof(candidates[0]), "%s", candidate);
      (*count)++;
    }
  }
  closedir(d);
  return true;
}

static int natural_compare(const char *a, const char *b) {
  while (*a && *b) {
    if (isdigit((unsigned char)*a) && isdigit((unsigned char)*b)) {
      const char *a_digits = a;
      const char *b_digits = b;
      while (isdigit((unsigned char)*a)) {
        a++;
      }
      while (isdigit((unsigned char)*b)) {
        b++;
      }
      size_t a_len = (size_t)(a - a_digits);
      size_t b_len = (size_t)(b - b_digits);
      while (a_len > 1 && *a_digits == '0') {
        a_digits++;
        a_len--;
      }
      while (b_len > 1 && *b_digits == '0') {
        b_digits++;
        b_len--;
      }
      if (a_len != b_len) {
        return a_len < b_len ? -1 : 1;
      }
      int cmp = strncmp(a_digits, b_digits, a_len);
      if (cmp != 0) {
        return cmp;
      }
    } else {
      if (*a != *b) {
        return (unsigned char)*a - (unsigned char)*b;
      }
      a++;
      b++;
    }
  }
  return (unsigned char)*a - (unsigned char)*b;
}

static int compare_strings_desc(const void *a, const void *b) {
  return natural_compare((const char *)b, (const char *)a);
}

bool wp_proton_bin(char *out, size_t out_len, char *err_out,
                   size_t err_out_len) {
  const char *override = getenv("WALLPIPER_PROTON_BIN");
  if (override && override[0] != '\0') {
    return fmt_ok(out, out_len, "%s", override);
  }

  char steam_root[768];
  if (!wp_steam_root(steam_root, sizeof(steam_root), err_out, err_out_len)) {
    return false;
  }

  char tools_dir[900];
  char common_dir[900];
  snprintf(tools_dir, sizeof(tools_dir), "%s/compatibilitytools.d", steam_root);
  snprintf(common_dir, sizeof(common_dir), "%s/steamapps/common", steam_root);

  char candidates[64][1024];
  size_t count = 0;
  scan_proton_candidates(tools_dir, candidates, 64, &count);
  scan_proton_candidates(common_dir, candidates, 64, &count);

  if (count == 0) {
    snprintf(err_out, err_out_len,
             "no Proton build found under %s or %s. Install Proton through "
             "Steam, or set WALLPIPER_PROTON_BIN to a "
             "proton binary's path",
             tools_dir, common_dir);
    return false;
  }

  qsort(candidates, count, sizeof(candidates[0]), compare_strings_desc);
  return fmt_ok(out, out_len, "%s", candidates[0]);
}

bool wp_wine_bin(char *out, size_t out_len, char *err_out, size_t err_out_len) {
  const char *override = getenv("WALLPIPER_WINE_BIN");
  if (override && override[0] != '\0') {
    return fmt_ok(out, out_len, "%s", override);
  }

  char proton[1024];
  if (!wp_proton_bin(proton, sizeof(proton), err_out, err_out_len)) {
    return false;
  }

  char *slash = strrchr(proton, '/');
  if (!slash) {
    snprintf(err_out, err_out_len,
             "could not derive wine binary from proton path %s", proton);
    return false;
  }
  *slash = '\0';

  char path[1024];
  if (!fmt_ok(path, sizeof(path), "%s/files/bin/wine", proton)) {
    snprintf(err_out, err_out_len, "proton path too long");
    return false;
  }
  if (!is_regular_file(path)) {
    snprintf(err_out, err_out_len,
             "wine binary not found at %s. set WALLPIPER_WINE_BIN to its path",
             path);
    return false;
  }
  return fmt_ok(out, out_len, "%s", path);
}

bool wp_portal_name(char *out, size_t out_len, char *err_out,
                    size_t err_out_len) {
  const char *portal = getenv("WALLPIPER_PORTAL");
  if (!portal) {
    snprintf(err_out, err_out_len,
             "WALLPIPER_PORTAL not set. export WALLPIPER_PORTAL=<name> naming "
             "an installed "
             "wallpiper-portal-<name> binary (e.g. WALLPIPER_PORTAL=hyprland)");
    return false;
  }
  return fmt_ok(out, out_len, "%s", portal);
}

bool wp_to_windows_path(const char *unix_path, char *out, size_t out_len) {
  size_t len = strlen(unix_path);
  if (len + 3 > out_len) {
    return false;
  }
  out[0] = 'Z';
  out[1] = ':';
  for (size_t i = 0; i < len; i++) {
    out[2 + i] = unix_path[i] == '/' ? '\\' : unix_path[i];
  }
  out[2 + len] = '\0';
  return true;
}

bool wp_from_windows_path(const char *windows_path, char *out, size_t out_len) {
  if (strncasecmp(windows_path, "Z:", 2) != 0) {
    return false;
  }
  const char *rest = windows_path + 2;
  size_t len = strlen(rest);
  if (len >= out_len) {
    return false;
  }
  for (size_t i = 0; i < len; i++) {
    out[i] = rest[i] == '\\' ? '/' : rest[i];
  }
  out[len] = '\0';
  return true;
}

static void report(const char *label, bool ok, const char *value_or_err) {
  if (ok) {
    printf("  %s: %s\n", label, value_or_err);
  } else {
    printf("  %s: ERROR %s\n", label, value_or_err);
  }
}

void wp_describe(void) {
  char buf[1024];
  char err[512];

  printf("wallpiper configuration:\n");

  bool ok = wp_install_dir(buf, sizeof(buf));
  report("install dir", ok,
         ok ? buf : "could not resolve wallpiperctl's own executable path");

  ok = wp_steam_root(buf, sizeof(buf), err, sizeof(err));
  report("steam root", ok, ok ? buf : err);

  ok = wp_proton_bin(buf, sizeof(buf), err, sizeof(err));
  report("proton binary", ok, ok ? buf : err);

  ok = wp_we_exe(buf, sizeof(buf), err, sizeof(err));
  report("wallpaper engine exe", ok, ok ? buf : err);

  ok = wp_wine_bin(buf, sizeof(buf), err, sizeof(err));
  report("wine binary", ok, ok ? buf : err);

  ok = wp_we_config_path(buf, sizeof(buf), err, sizeof(err));
  report("wallpaper engine config.json", ok, ok ? buf : err);

  ok = wp_compatdata_dir(buf, sizeof(buf), err, sizeof(err));
  report("compatdata dir", ok, ok ? buf : err);

  ok = wp_workshop_content_dir(buf, sizeof(buf), err, sizeof(err));
  report("workshop content dir", ok, ok ? buf : err);

  wp_temp_dir(buf, sizeof(buf));
  report("temp dir (WALLPIPER_TEMP_DIR)", true, buf);

  wp_runtime_dir(buf, sizeof(buf));
  report("runtime dir (WALLPIPER_RUNTIME_DIR)", true, buf);

  ok = wp_portal_name(buf, sizeof(buf), err, sizeof(err));
  report("portal (WALLPIPER_PORTAL)", ok, ok ? buf : err);
}
