#pragma once

#define HAVE_EGL 1
#include "clutter/clutter.h"
#include <EGL/egl.h>
#include <EGL/eglext.h>

G_BEGIN_DECLS
void wallpiper_egl_wait_sync_fd(EGLDisplay egl_display, int sync_fd);
CoglTexture *wallpiper_egl_import_dmabuf(CoglContext *cogl_context,
                                         EGLDisplay egl_display, int fd,
                                         uint32_t width, uint32_t height,
                                         uint32_t stride, uint32_t offset,
                                         uint64_t modifier, GError **error);
G_END_DECLS
