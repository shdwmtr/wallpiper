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

#include "egl_import.h"
#include "error.h"

#include <EGL/eglext.h>
#include <drm_fourcc.h>
#include <unistd.h>

typedef EGLImageKHR (*EglCreateImageKHRFunc)(EGLDisplay, EGLContext, EGLenum,
                                             EGLClientBuffer, const EGLint *);
typedef EGLBoolean (*EglDestroyImageKHRFunc)(EGLDisplay, EGLImageKHR);
typedef EGLSyncKHR (*EglCreateSyncKHRFunc)(EGLDisplay, EGLenum, const EGLint *);
typedef EGLBoolean (*EglDestroySyncKHRFunc)(EGLDisplay, EGLSyncKHR);
typedef EGLint (*EglClientWaitSyncKHRFunc)(EGLDisplay, EGLSyncKHR, EGLint,
                                           EGLTimeKHR);

static EglCreateImageKHRFunc eglCreateImageKHR_ = NULL;
static EglDestroyImageKHRFunc eglDestroyImageKHR_ = NULL;
static EglCreateSyncKHRFunc eglCreateSyncKHR_ = NULL;
static EglDestroySyncKHRFunc eglDestroySyncKHR_ = NULL;
static EglClientWaitSyncKHRFunc eglClientWaitSyncKHR_ = NULL;

static void ensure_egl_funcs(void) {
  if (!eglCreateImageKHR_)
    eglCreateImageKHR_ =
        (EglCreateImageKHRFunc)eglGetProcAddress("eglCreateImageKHR");
  if (!eglDestroyImageKHR_)
    eglDestroyImageKHR_ =
        (EglDestroyImageKHRFunc)eglGetProcAddress("eglDestroyImageKHR");
  if (!eglCreateSyncKHR_)
    eglCreateSyncKHR_ =
        (EglCreateSyncKHRFunc)eglGetProcAddress("eglCreateSyncKHR");
  if (!eglDestroySyncKHR_)
    eglDestroySyncKHR_ =
        (EglDestroySyncKHRFunc)eglGetProcAddress("eglDestroySyncKHR");
  if (!eglClientWaitSyncKHR_)
    eglClientWaitSyncKHR_ =
        (EglClientWaitSyncKHRFunc)eglGetProcAddress("eglClientWaitSyncKHR");
}

void wallpiper_egl_wait_sync_fd(EGLDisplay egl_display, int sync_fd) {
  if (sync_fd < 0)
    return;

  ensure_egl_funcs();

  if (!eglCreateSyncKHR_ || !eglClientWaitSyncKHR_) {
    close(sync_fd);
    return;
  }

  EGLint attribs[] = {EGL_SYNC_NATIVE_FENCE_FD_ANDROID, sync_fd, EGL_NONE};
  EGLSyncKHR sync =
      eglCreateSyncKHR_(egl_display, EGL_SYNC_NATIVE_FENCE_ANDROID, attribs);
  if (sync == EGL_NO_SYNC_KHR) {
    close(sync_fd);
    return;
  }

  eglClientWaitSyncKHR_(egl_display, sync, 0, EGL_FOREVER_KHR);

  if (eglDestroySyncKHR_)
    eglDestroySyncKHR_(egl_display, sync);
}

// some magic based on:
// https://github.com/GNOME/mutter/blob/main/src/backends/native/meta-renderer-native.c

CoglTexture *wallpiper_egl_import_dmabuf(CoglContext *cogl_context,
                                         EGLDisplay egl_display, int fd,
                                         uint32_t width, uint32_t height,
                                         uint32_t stride, uint32_t offset,
                                         uint64_t modifier, GError **error) {
  ensure_egl_funcs();

  EGLint attribs[32];
  int i = 0;
  attribs[i++] = EGL_WIDTH;
  attribs[i++] = (EGLint)width;
  attribs[i++] = EGL_HEIGHT;
  attribs[i++] = (EGLint)height;
  attribs[i++] = EGL_LINUX_DRM_FOURCC_EXT;
  attribs[i++] = DRM_FORMAT_XRGB8888;
  attribs[i++] = EGL_DMA_BUF_PLANE0_FD_EXT;
  attribs[i++] = fd;
  attribs[i++] = EGL_DMA_BUF_PLANE0_OFFSET_EXT;
  attribs[i++] = (EGLint)offset;
  attribs[i++] = EGL_DMA_BUF_PLANE0_PITCH_EXT;
  attribs[i++] = (EGLint)stride;
  if (modifier != DRM_FORMAT_MOD_INVALID) {
    attribs[i++] = EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT;
    attribs[i++] = (EGLint)(modifier & 0xffffffff);
    attribs[i++] = EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT;
    attribs[i++] = (EGLint)(modifier >> 32);
  }
  attribs[i++] = EGL_NONE;

  EGLImageKHR image = eglCreateImageKHR_(egl_display, EGL_NO_CONTEXT,
                                         EGL_LINUX_DMA_BUF_EXT, NULL, attribs);
  if (image == EGL_NO_IMAGE_KHR) {
    g_set_error(error, WALLPIPER_ERROR, 0, "eglCreateImageKHR failed: 0x%x",
                eglGetError());
    return NULL;
  }

  GError *texture_error = NULL;
  CoglTexture *texture = cogl_texture_2d_new_from_egl_image(
      cogl_context, (int)width, (int)height, COGL_PIXEL_FORMAT_BGRX_8888, image,
      COGL_EGL_IMAGE_FLAG_NONE, &texture_error);

  eglDestroyImageKHR_(egl_display, image);

  if (!texture) {
    g_propagate_error(error, texture_error);
    return NULL;
  }

  return texture;
}
