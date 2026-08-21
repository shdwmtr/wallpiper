#pragma once

#include <stdbool.h>
#include <stddef.h>

typedef struct {
  char id[128];
  char title[256];
  char kind[64];
} wp_wallpaper_info_t;

typedef struct {
  char key[128];
  char kind[64];
  char text[512];
  char value_json[2048];
} wp_property_info_t;

bool wp_wallpaper_catalog_list(const char *workshop_content_dir,
                               wp_wallpaper_info_t *out, size_t max_out,
                               size_t *out_count);

bool wp_wallpaper_catalog_properties(const char *workshop_content_dir,
                                     const char *id, char *title_out,
                                     size_t title_out_len,
                                     wp_property_info_t *out, size_t max_out,
                                     size_t *out_count);

bool wp_wallpaper_catalog_resolve(const char *workshop_content_dir,
                                  const char *id, char *err_out,
                                  size_t err_out_len);
