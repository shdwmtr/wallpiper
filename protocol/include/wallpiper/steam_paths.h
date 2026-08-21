#pragma once

#include <stdbool.h>
#include <stddef.h>

#define WALLPAPER_ENGINE_APP_ID "431960"

bool wp_home_dir(char *out, size_t out_len);
bool wp_install_dir(char *out, size_t out_len);

bool wp_steam_root(char *out, size_t out_len, char *err_out,
                   size_t err_out_len);
bool wp_location_slug(const char *location, char *out, size_t out_len);
bool wp_compatdata_for(const char *location, char *out, size_t out_len,
                       char *err_out, size_t err_out_len);
bool wp_workshop_content_dir(char *out, size_t out_len, char *err_out,
                             size_t err_out_len);
bool wp_we_config_path(char *out, size_t out_len, char *err_out,
                       size_t err_out_len);
bool wp_we_exe(char *out, size_t out_len, char *err_out, size_t err_out_len);
bool wp_proton_bin(char *out, size_t out_len, char *err_out,
                   size_t err_out_len);
bool wp_portal_name(char *out, size_t out_len, char *err_out,
                    size_t err_out_len);

bool wp_to_windows_path(const char *unix_path, char *out, size_t out_len);
bool wp_from_windows_path(const char *windows_path, char *out, size_t out_len);

void wp_describe(void);
