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

#include <stdint.h>

#define WP_VK_FORMAT_R8G8B8A8_UNORM 37u
#define WP_VK_FORMAT_R8G8B8A8_SRGB 43u
#define WP_VK_FORMAT_B8G8R8A8_UNORM 44u
#define WP_VK_FORMAT_B8G8R8A8_SRGB 50u
#define WP_VK_FORMAT_A8B8G8R8_UNORM_PACK32 51u
#define WP_VK_FORMAT_A8B8G8R8_SRGB_PACK32 57u

#define WP_DRM_FORMAT_XRGB8888 0x34325258u
#define WP_DRM_FORMAT_XBGR8888 0x34324258u

static inline uint32_t wp_drm_fourcc_from_vk_format(uint32_t vk_format,
                                                     int *out_matched) {
  switch (vk_format) {
  case WP_VK_FORMAT_B8G8R8A8_UNORM:
  case WP_VK_FORMAT_B8G8R8A8_SRGB:
    if (out_matched) {
      *out_matched = 1;
    }
    return WP_DRM_FORMAT_XRGB8888;
  case WP_VK_FORMAT_R8G8B8A8_UNORM:
  case WP_VK_FORMAT_R8G8B8A8_SRGB:
  case WP_VK_FORMAT_A8B8G8R8_UNORM_PACK32:
  case WP_VK_FORMAT_A8B8G8R8_SRGB_PACK32:
    if (out_matched) {
      *out_matched = 1;
    }
    return WP_DRM_FORMAT_XBGR8888;
  default:
    if (out_matched) {
      *out_matched = 0;
    }
    return WP_DRM_FORMAT_XRGB8888;
  }
}
