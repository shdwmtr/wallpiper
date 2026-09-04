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

#include "state_internal.h"

#include <wallpiper/vk_format.h>

#include <stdio.h>

struct wl_buffer *wp_wl_create_dmabuf_buffer(wp_wl_state_t *state, int fd,
                                             uint32_t width, uint32_t height,
                                             uint32_t format, uint32_t stride,
                                             uint64_t modifier) {
  if (!state->dmabuf) {
    printf("zwp_linux_dmabuf_v1 not available\n");
    return NULL;
  }

  struct zwp_linux_buffer_params_v1 *params =
      zwp_linux_dmabuf_v1_create_params(state->dmabuf);
  if (!params) {
    return NULL;
  }

  uint32_t modifier_hi = (uint32_t)(modifier >> 32);
  uint32_t modifier_lo = (uint32_t)(modifier & 0xffffffffu);
  zwp_linux_buffer_params_v1_add(params, fd, 0, 0, stride, modifier_hi,
                                 modifier_lo);

  uint32_t fourcc = wp_drm_fourcc_from_vk_format(format, NULL);
  struct wl_buffer *buffer = zwp_linux_buffer_params_v1_create_immed(
      params, (int32_t)width, (int32_t)height, fourcc, 0);
  zwp_linux_buffer_params_v1_destroy(params);

  return buffer;
}
