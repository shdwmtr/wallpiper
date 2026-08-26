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
