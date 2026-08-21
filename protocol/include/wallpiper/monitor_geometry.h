#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct {
  int32_t x;
  int32_t y;
  uint32_t width;
  uint32_t height;
  uint32_t logical_width;
  uint32_t logical_height;
  double scale;
} wp_monitor_geometry_t;

bool wp_monitor_geometry_encode_json(const wp_monitor_geometry_t *geometry,
                                     char *out, size_t out_len);
bool wp_monitor_geometry_decode_json(const char *json,
                                     wp_monitor_geometry_t *out);
