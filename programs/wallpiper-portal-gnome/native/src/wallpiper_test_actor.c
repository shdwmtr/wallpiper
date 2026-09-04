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

#include "wallpiper_portal.h"

#include "egl_import.h"
#include "error.h"
#include "mutter_private.h"

#include <wallpiper/vk_format.h>

#include <gbm.h>

#include <errno.h>
#include <fcntl.h>
#include <unistd.h>

#define TEST_WIDTH 512
#define TEST_HEIGHT 512

static struct gbm_device *open_render_node(int *out_fd) {
  static const char *candidates[] = {
      "/dev/dri/renderD128",
      "/dev/dri/renderD129",
      "/dev/dri/renderD130",
      NULL,
  };

  for (int i = 0; candidates[i]; i++) {
    int fd = open(candidates[i], O_RDWR | O_CLOEXEC);
    if (fd < 0)
      continue;

    struct gbm_device *dev = gbm_create_device(fd);
    if (dev) {
      *out_fd = fd;
      return dev;
    }

    close(fd);
  }

  return NULL;
}

static void fill_test_pattern(struct gbm_bo *bo) {
  uint32_t stride = 0;
  void *map_data = NULL;
  uint8_t *pixels = gbm_bo_map(bo, 0, 0, TEST_WIDTH, TEST_HEIGHT,
                               GBM_BO_TRANSFER_WRITE, &stride, &map_data);
  if (!pixels)
    return;

  for (int y = 0; y < TEST_HEIGHT; y++) {
    uint32_t *row = (uint32_t *)(pixels + y * stride);
    for (int x = 0; x < TEST_WIDTH; x++) {
      gboolean checker = ((x / 32) + (y / 32)) % 2 == 0;
      uint8_t r = checker ? 255 : (uint8_t)(255 * x / TEST_WIDTH);
      uint8_t g = checker ? 0 : (uint8_t)(255 * y / TEST_HEIGHT);
      uint8_t b = checker ? 128 : 255;
      row[x] = (0xffu << 24) | ((uint32_t)r << 16) | ((uint32_t)g << 8) | b;
    }
  }

  gbm_bo_unmap(bo, map_data);
}

gboolean wallpiper_place_test_actor(GObject *backend_obj, GObject *parent_obj,
                                    GObject *below_sibling_obj, int x, int y,
                                    int width, int height, GError **error) {
  if (!META_IS_BACKEND(backend_obj)) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "backend argument is not a MetaBackend");
    return FALSE;
  }

  if (!CLUTTER_IS_ACTOR(parent_obj)) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "parent argument is not a ClutterActor");
    return FALSE;
  }

  if (below_sibling_obj && !CLUTTER_IS_ACTOR(below_sibling_obj)) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "below_sibling argument is not a ClutterActor");
    return FALSE;
  }

  ClutterActor *parent = CLUTTER_ACTOR(parent_obj);
  ClutterActor *below_sibling =
      below_sibling_obj ? CLUTTER_ACTOR(below_sibling_obj) : NULL;
  MetaBackend *backend = META_BACKEND(backend_obj);
  ClutterBackend *clutter_backend = meta_backend_get_clutter_backend(backend);
  CoglContext *cogl_context = clutter_backend_get_cogl_context(clutter_backend);
  EGLDisplay egl_display = cogl_context_get_egl_display(cogl_context);

  g_message("wallpiper-gnome: got cogl_context=%p egl_display=%p",
            (void *)cogl_context, (void *)egl_display);

  if (!egl_display) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "could not get Mutter's EGLDisplay from Cogl");
    return FALSE;
  }

  int render_fd = -1;
  struct gbm_device *gbm_dev = open_render_node(&render_fd);
  if (!gbm_dev) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "could not open a DRM render node for GBM");
    return FALSE;
  }

  struct gbm_bo *bo =
      gbm_bo_create(gbm_dev, TEST_WIDTH, TEST_HEIGHT, GBM_FORMAT_XRGB8888,
                    GBM_BO_USE_LINEAR | GBM_BO_USE_RENDERING);
  if (!bo) {
    int saved_errno = errno;
    g_message("wallpiper-gnome: gbm_bo_create(LINEAR|RENDERING) failed: %s, "
              "retrying with RENDERING only",
              g_strerror(saved_errno));
    bo = gbm_bo_create(gbm_dev, TEST_WIDTH, TEST_HEIGHT, GBM_FORMAT_XRGB8888,
                       GBM_BO_USE_RENDERING);
  }
  if (!bo) {
    int saved_errno = errno;
    g_message("wallpiper-gnome: gbm_bo_create(RENDERING) failed: %s, retrying "
              "with SCANOUT|RENDERING",
              g_strerror(saved_errno));
    bo = gbm_bo_create(gbm_dev, TEST_WIDTH, TEST_HEIGHT, GBM_FORMAT_XRGB8888,
                       GBM_BO_USE_SCANOUT | GBM_BO_USE_RENDERING);
  }
  if (!bo) {
    int saved_errno = errno;
    g_set_error(error, WALLPIPER_ERROR, 0,
                "gbm_bo_create failed in every flag combination tried: %s",
                g_strerror(saved_errno));
    gbm_device_destroy(gbm_dev);
    close(render_fd);
    return FALSE;
  }

  fill_test_pattern(bo);

  int dmabuf_fd = gbm_bo_get_fd(bo);
  uint32_t stride = gbm_bo_get_stride(bo);
  uint32_t offset = gbm_bo_get_offset(bo, 0);
  uint64_t modifier = gbm_bo_get_modifier(bo);

  g_message("wallpiper-gnome: dmabuf fd=%d stride=%u offset=%u modifier=0x%llx",
            dmabuf_fd, stride, offset, (unsigned long long)modifier);

  CoglTexture *texture = wallpiper_egl_import_dmabuf(
      cogl_context, egl_display, dmabuf_fd, TEST_WIDTH, TEST_HEIGHT,
      WP_VK_FORMAT_B8G8R8A8_UNORM, stride, offset, modifier, error);

  close(dmabuf_fd);
  gbm_bo_destroy(bo);
  gbm_device_destroy(gbm_dev);
  close(render_fd);

  if (!texture)
    return FALSE;

  g_message("wallpiper-gnome: CoglTexture2D created at %p, wrapping as "
            "ClutterContent",
            (void *)texture);

  ClutterContent *content =
      clutter_texture_content_new_from_texture(texture, NULL);

  ClutterActor *actor = clutter_actor_new();
  clutter_actor_set_position(actor, x, y);
  clutter_actor_set_size(actor, width, height);
  clutter_actor_set_content(actor, content);
  clutter_actor_set_content_gravity(actor, CLUTTER_CONTENT_GRAVITY_RESIZE_FILL);
  clutter_actor_show(actor);

  if (below_sibling)
    clutter_actor_insert_child_below(parent, actor, below_sibling);
  else
    clutter_actor_add_child(parent, actor);

  g_message("wallpiper-gnome: actor placed at (%d,%d) %dx%d under parent %p "
            "(below_sibling=%p)",
            x, y, width, height, (void *)parent, (void *)below_sibling);

  return TRUE;
}
