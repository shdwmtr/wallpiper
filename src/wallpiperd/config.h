#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "wallpiper/monitor_geometry.h"

extern const wp_monitor_geometry_t WP_FALLBACK_MONITOR;

bool wp_dpi_marker_path(const char *location, char *out, size_t out_len);
bool wp_renderer_pid_path(const char *location, char *out, size_t out_len);
bool wp_tray_icon_path(char *out, size_t out_len);
bool wp_tray_click_path(char *out, size_t out_len);
bool wp_menu_file_path(char *out, size_t out_len);
bool wp_menu_command_path(char *out, size_t out_len);
bool wp_cursor_pos_path(const char *location, char *out, size_t out_len);
bool wp_selectwallpaper_request_path(const char *location, char *out,
                                     size_t out_len);
bool wp_selectwallpaper_reply_path(const char *location, char *out,
                                   size_t out_len);
bool wp_wine_fonts_dir(const char *location, char *out, size_t out_len);
bool wp_font_cache_dir(char *out, size_t out_len);
bool wp_preload_path(char *out, size_t out_len);
bool wp_dwmapi_shim_path(char *out, size_t out_len);
bool wp_wine_builtin_dwmapi_path(char *out, size_t out_len);
bool wp_vk_layer_path(char *out, size_t out_len);
