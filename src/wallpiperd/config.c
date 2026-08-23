#include "config.h"

#include <stdarg.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

#include "wallpiper/protocol.h"
#include "wallpiper/steam_paths.h"

const wp_monitor_geometry_t WP_FALLBACK_MONITOR = {
    .x = 0,
    .y = 0,
    .width = 1920,
    .height = 1080,
    .logical_width = 1920,
    .logical_height = 1080,
    .scale = 1.0,
};

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
