#include "wallpiper/debug_overlay.h"

#include <math.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define WP_STATS_WINDOW_SECONDS 3.0
#define WP_SPARKLINE_SAMPLES 40
#define WP_FRAME_STATS_CAPACITY 4096

static const uint8_t COLOR_BG[4] = {20, 20, 24, 200};
static const uint8_t COLOR_PRIMARY[4] = {255, 255, 255, 255};
static const uint8_t COLOR_SECONDARY[4] = {150, 150, 150, 255};
static const uint8_t COLOR_BAR_OK[4] = {130, 190, 255, 255};
static const uint8_t COLOR_BAR_WARN[4] = {255, 90, 70, 255};

typedef struct {
  double times[WP_FRAME_STATS_CAPACITY];
  size_t head;
  size_t count;
} frame_stat_ring_t;

struct wp_frame_stats {
  frame_stat_ring_t display;
  frame_stat_ring_t capture;
};

static double now_seconds(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
}

static void ring_record(frame_stat_ring_t *ring) {
  double now = now_seconds();

  if (ring->count == WP_FRAME_STATS_CAPACITY) {
    ring->head = (ring->head + 1) % WP_FRAME_STATS_CAPACITY;
    ring->count--;
  }
  size_t idx = (ring->head + ring->count) % WP_FRAME_STATS_CAPACITY;
  ring->times[idx] = now;
  ring->count++;

  while (ring->count > 0 &&
         (now - ring->times[ring->head]) > WP_STATS_WINDOW_SECONDS) {
    ring->head = (ring->head + 1) % WP_FRAME_STATS_CAPACITY;
    ring->count--;
  }
}

static size_t ring_fps_in_last_second(const frame_stat_ring_t *ring) {
  double now = now_seconds();
  size_t n = 0;
  for (size_t i = 0; i < ring->count; i++) {
    size_t idx = (ring->head + i) % WP_FRAME_STATS_CAPACITY;
    if (now - ring->times[idx] <= 1.0) {
      n++;
    }
  }
  return n;
}

wp_frame_stats_t *wp_frame_stats_create(void) {
  return calloc(1, sizeof(wp_frame_stats_t));
}

void wp_frame_stats_destroy(wp_frame_stats_t *stats) { free(stats); }

void wp_frame_stats_record_display(wp_frame_stats_t *stats) {
  ring_record(&stats->display);
}

void wp_frame_stats_record_capture(wp_frame_stats_t *stats) {
  ring_record(&stats->capture);
}

void wp_debug_throttle_init(wp_debug_throttle_t *throttle) {
  throttle->has_last_draw = false;
  throttle->last_draw_seconds = 0.0;
}

void wp_debug_throttle_reset(wp_debug_throttle_t *throttle) {
  throttle->has_last_draw = false;
}

bool wp_debug_throttle_should_redraw(wp_debug_throttle_t *throttle) {
  double now = now_seconds();
  if (throttle->has_last_draw && (now - throttle->last_draw_seconds) * 1000.0 <
                                     WP_DEBUG_OVERLAY_REDRAW_INTERVAL_MS) {
    return false;
  }
  throttle->has_last_draw = true;
  throttle->last_draw_seconds = now;
  return true;
}

static void put_pixel(uint8_t *pixels, int stride, int buf_w, int buf_h, int x,
                      int y, const uint8_t color[4]) {
  if (x < 0 || y < 0 || x >= buf_w || y >= buf_h) {
    return;
  }
  size_t offset = (size_t)y * (size_t)stride + (size_t)x * 4;
  pixels[offset] = color[2];
  pixels[offset + 1] = color[1];
  pixels[offset + 2] = color[0];
  pixels[offset + 3] = color[3];
}

static void fill_rect(uint8_t *pixels, int stride, int buf_w, int buf_h, int x,
                      int y, int w, int h, const uint8_t color[4]) {
  for (int row = y; row < y + h; row++) {
    for (int col = x; col < x + w; col++) {
      put_pixel(pixels, stride, buf_w, buf_h, col, row, color);
    }
  }
}

static void fill_bg(uint8_t *pixels, int stride, int buf_w, int buf_h) {
  fill_rect(pixels, stride, buf_w, buf_h, 0, 0, buf_w, buf_h, COLOR_BG);
}

static const uint8_t SEGMENTS[10] = {
    0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f,
};

static void draw_digit(uint8_t *pixels, int stride, int buf_w, int buf_h, int x,
                       int y, uint8_t digit, int cell_w, int cell_h, int thick,
                       const uint8_t color[4]) {
  uint8_t bits = SEGMENTS[digit % 10];
  int half = cell_h / 2;

  if (bits & 0x01) {
    fill_rect(pixels, stride, buf_w, buf_h, x, y, cell_w, thick, color);
  }
  if (bits & 0x02) {
    fill_rect(pixels, stride, buf_w, buf_h, x + cell_w - thick, y, thick, half,
              color);
  }
  if (bits & 0x04) {
    fill_rect(pixels, stride, buf_w, buf_h, x + cell_w - thick, y + half, thick,
              half, color);
  }
  if (bits & 0x08) {
    fill_rect(pixels, stride, buf_w, buf_h, x, y + cell_h - thick, cell_w,
              thick, color);
  }
  if (bits & 0x10) {
    fill_rect(pixels, stride, buf_w, buf_h, x, y + half, thick, half, color);
  }
  if (bits & 0x20) {
    fill_rect(pixels, stride, buf_w, buf_h, x, y, thick, half, color);
  }
  if (bits & 0x40) {
    fill_rect(pixels, stride, buf_w, buf_h, x, y + half - thick / 2, cell_w,
              thick, color);
  }
}

static int draw_number(uint8_t *pixels, int stride, int buf_w, int buf_h, int x,
                       int y, uint32_t value, int cell_w, int cell_h, int thick,
                       const uint8_t color[4]) {
  if (value > 999) {
    value = 999;
  }

  uint8_t digits[8];
  int digit_count = 0;
  if (value == 0) {
    digits[digit_count++] = 0;
  } else {
    uint32_t v = value;
    while (v > 0) {
      digits[digit_count++] = (uint8_t)(v % 10);
      v /= 10;
    }
    for (int i = 0; i < digit_count / 2; i++) {
      uint8_t tmp = digits[i];
      digits[i] = digits[digit_count - 1 - i];
      digits[digit_count - 1 - i] = tmp;
    }
  }

  int gap = thick;
  int cursor = x;
  for (int i = 0; i < digit_count; i++) {
    draw_digit(pixels, stride, buf_w, buf_h, cursor, y, digits[i], cell_w,
               cell_h, thick, color);
    cursor += cell_w + gap;
  }
  return cursor - x;
}

static bool glyph_5x5(char ch, uint8_t rows[5]) {
  static const uint8_t glyph_d[5] = {0x1c, 0x12, 0x11, 0x12, 0x1c};
  static const uint8_t glyph_c[5] = {0x0f, 0x10, 0x10, 0x10, 0x0f};
  static const uint8_t glyph_f[5] = {0x1f, 0x10, 0x1e, 0x10, 0x10};

  const uint8_t *src;
  switch (ch) {
  case 'D':
    src = glyph_d;
    break;
  case 'C':
    src = glyph_c;
    break;
  case 'F':
    src = glyph_f;
    break;
  default:
    return false;
  }
  memcpy(rows, src, 5);
  return true;
}

static void draw_letter(uint8_t *pixels, int stride, int buf_w, int buf_h,
                        int x, int y, char ch, int scale,
                        const uint8_t color[4]) {
  uint8_t rows[5];
  if (!glyph_5x5(ch, rows)) {
    return;
  }
  for (int row_idx = 0; row_idx < 5; row_idx++) {
    uint8_t row_bits = rows[row_idx];
    for (int col_idx = 0; col_idx < 5; col_idx++) {
      if (row_bits & (1 << (4 - col_idx))) {
        fill_rect(pixels, stride, buf_w, buf_h, x + col_idx * scale,
                  y + row_idx * scale, scale, scale, color);
      }
    }
  }
}

static void draw_row(uint8_t *pixels, int stride, int buf_w, int buf_h, int x,
                     int y, char label, uint32_t primary_value,
                     bool has_secondary, uint32_t secondary_value) {
  draw_letter(pixels, stride, buf_w, buf_h, x, y + 2, label, 3, COLOR_PRIMARY);
  int number_x = x + 22;
  int consumed = draw_number(pixels, stride, buf_w, buf_h, number_x, y,
                             primary_value, 12, 20, 3, COLOR_PRIMARY);
  if (has_secondary) {
    draw_number(pixels, stride, buf_w, buf_h, number_x + consumed + 10, y + 7,
                secondary_value, 8, 13, 2, COLOR_SECONDARY);
  }
}

static void draw_sparkline(uint8_t *pixels, int stride, int buf_w, int buf_h,
                           int x, int y, int w, int h, const double *values,
                           size_t value_count, double warn_threshold_ms) {
  const double MAX_SCALE_MS = 40.0;

  int slot_w = w / WP_SPARKLINE_SAMPLES;
  if (slot_w < 1) {
    slot_w = 1;
  }

  for (size_t i = 0; i < value_count; i++) {
    double delta_ms = values[i];
    double frac = delta_ms / MAX_SCALE_MS;
    if (frac < 0.02) {
      frac = 0.02;
    }
    if (frac > 1.0) {
      frac = 1.0;
    }
    int bar_h = (int)(frac * (double)h);
    const uint8_t *color =
        delta_ms > warn_threshold_ms ? COLOR_BAR_WARN : COLOR_BAR_OK;
    int bar_x = x + (int)i * slot_w;
    int bar_w = slot_w - 1;
    if (bar_w < 1) {
      bar_w = 1;
    }
    fill_rect(pixels, stride, buf_w, buf_h, bar_x, y + h - bar_h, bar_w, bar_h,
              color);
  }
}

void wp_render_stats_panel(const wp_frame_stats_t *stats,
                           uint8_t out[WP_DEBUG_OVERLAY_BUFFER_SIZE]) {
  memset(out, 0, WP_DEBUG_OVERLAY_BUFFER_SIZE);

  size_t display_fps = ring_fps_in_last_second(&stats->display);
  size_t capture_fps = ring_fps_in_last_second(&stats->capture);

  double deltas_ms[WP_FRAME_STATS_CAPACITY];
  size_t delta_count = 0;
  for (size_t i = 1; i < stats->display.count; i++) {
    size_t prev_idx = (stats->display.head + i - 1) % WP_FRAME_STATS_CAPACITY;
    size_t cur_idx = (stats->display.head + i) % WP_FRAME_STATS_CAPACITY;
    double dt =
        (stats->display.times[cur_idx] - stats->display.times[prev_idx]) *
        1000.0;
    deltas_ms[delta_count++] = dt;
  }

  double last_ms = delta_count > 0 ? deltas_ms[delta_count - 1] : 0.0;
  double peak_ms = 0.0;
  for (size_t i = 0; i < delta_count; i++) {
    if (deltas_ms[i] > peak_ms) {
      peak_ms = deltas_ms[i];
    }
  }

  size_t sparkline_count =
      delta_count < WP_SPARKLINE_SAMPLES ? delta_count : WP_SPARKLINE_SAMPLES;
  const double *sparkline = deltas_ms + (delta_count - sparkline_count);

  int bw = WP_DEBUG_OVERLAY_WIDTH;
  int bh = WP_DEBUG_OVERLAY_HEIGHT;
  int stride = WP_DEBUG_OVERLAY_STRIDE;

  fill_bg(out, stride, bw, bh);
  draw_row(out, stride, bw, bh, 10, 8, 'D', (uint32_t)display_fps, false, 0);
  draw_row(out, stride, bw, bh, 10, 40, 'C', (uint32_t)capture_fps, false, 0);
  draw_row(out, stride, bw, bh, 10, 72, 'F', (uint32_t)llround(last_ms), true,
           (uint32_t)llround(peak_ms));
  draw_sparkline(out, stride, bw, bh, 10, 100, bw - 20, 14, sparkline,
                 sparkline_count, 33.3);
}
