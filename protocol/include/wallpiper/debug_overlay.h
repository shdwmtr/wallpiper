#pragma once

#include <stdbool.h>
#include <stdint.h>

#define WP_DEBUG_OVERLAY_WIDTH 210
#define WP_DEBUG_OVERLAY_HEIGHT 120
#define WP_DEBUG_OVERLAY_STRIDE (WP_DEBUG_OVERLAY_WIDTH * 4)
#define WP_DEBUG_OVERLAY_BUFFER_SIZE                                           \
  (WP_DEBUG_OVERLAY_STRIDE * WP_DEBUG_OVERLAY_HEIGHT)
#define WP_DEBUG_OVERLAY_REDRAW_INTERVAL_MS 250

typedef struct wp_frame_stats wp_frame_stats_t;

wp_frame_stats_t *wp_frame_stats_create(void);
void wp_frame_stats_destroy(wp_frame_stats_t *stats);
void wp_frame_stats_record_display(wp_frame_stats_t *stats);
void wp_frame_stats_record_capture(wp_frame_stats_t *stats);

typedef struct {
  bool has_last_draw;
  double last_draw_seconds;
} wp_debug_throttle_t;

void wp_debug_throttle_init(wp_debug_throttle_t *throttle);
void wp_debug_throttle_reset(wp_debug_throttle_t *throttle);
bool wp_debug_throttle_should_redraw(wp_debug_throttle_t *throttle);

void wp_render_stats_panel(const wp_frame_stats_t *stats,
                           uint8_t out[WP_DEBUG_OVERLAY_BUFFER_SIZE]);
