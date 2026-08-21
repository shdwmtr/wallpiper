#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "wallpiper/ctl_protocol.h"
#include "wallpiper/monitor_geometry.h"

typedef bool (*wp_wl_try_geometry_fn)(wp_monitor_geometry_t *out);

typedef struct {
  const char *portal_name;
  const char *layer_namespace;
  wp_wl_try_geometry_fn try_geometry;
  wp_ctl_cursor_pos_fn cursor_pos;
  void *cursor_ctx;
} wp_wl_portal_config_t;

void wp_wl_portal_run(const wp_wl_portal_config_t *config);

void wp_wl_geometry_from_scale(int32_t x, int32_t y, uint32_t width,
                               uint32_t height, double scale,
                               wp_monitor_geometry_t *out);
