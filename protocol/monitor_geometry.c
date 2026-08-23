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

#include "wallpiper/monitor_geometry.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

bool wp_monitor_geometry_encode_json(const wp_monitor_geometry_t *geometry,
                                     char *out, size_t out_len) {
  char scale_buf[64];
  int scale_len =
      snprintf(scale_buf, sizeof(scale_buf), "%.6g", geometry->scale);
  if (scale_len <= 0 || (size_t)scale_len >= sizeof(scale_buf)) {
    return false;
  }
  if (!strchr(scale_buf, '.') && !strchr(scale_buf, 'e') &&
      !strchr(scale_buf, 'n')) {
    if ((size_t)scale_len + 2 >= sizeof(scale_buf)) {
      return false;
    }
    scale_buf[scale_len] = '.';
    scale_buf[scale_len + 1] = '0';
    scale_buf[scale_len + 2] = '\0';
  }

  int n =
      snprintf(out, out_len,
               "{\"x\":%d,\"y\":%d,\"width\":%u,\"height\":%u,"
               "\"logical_width\":%u,\"logical_height\":%u,\"scale\":%s}",
               geometry->x, geometry->y, geometry->width, geometry->height,
               geometry->logical_width, geometry->logical_height, scale_buf);
  return n > 0 && (size_t)n < out_len;
}

static bool find_number(const char *json, const char *key, double *out) {
  char pattern[40];
  int pn = snprintf(pattern, sizeof(pattern), "\"%s\"", key);
  if (pn <= 0 || (size_t)pn >= sizeof(pattern)) {
    return false;
  }

  const char *p = strstr(json, pattern);
  if (!p) {
    return false;
  }
  p += pn;
  while (*p == ' ' || *p == '\t') {
    p++;
  }
  if (*p != ':') {
    return false;
  }
  p++;
  while (*p == ' ' || *p == '\t') {
    p++;
  }

  char *end = NULL;
  double v = strtod(p, &end);
  if (end == p) {
    return false;
  }
  *out = v;
  return true;
}

bool wp_monitor_geometry_decode_json(const char *json,
                                     wp_monitor_geometry_t *out) {
  double x, y, width, height, logical_width, logical_height, scale;

  if (!find_number(json, "x", &x) || !find_number(json, "y", &y) ||
      !find_number(json, "width", &width) ||
      !find_number(json, "height", &height) ||
      !find_number(json, "logical_width", &logical_width) ||
      !find_number(json, "logical_height", &logical_height)) {
    return false;
  }
  if (!find_number(json, "scale", &scale)) {
    scale = 1.0;
  }

  out->x = (int32_t)x;
  out->y = (int32_t)y;
  out->width = (uint32_t)width;
  out->height = (uint32_t)height;
  out->logical_width = (uint32_t)logical_width;
  out->logical_height = (uint32_t)logical_height;
  out->scale = scale;
  return true;
}
