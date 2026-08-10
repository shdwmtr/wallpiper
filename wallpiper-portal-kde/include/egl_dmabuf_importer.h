#pragma once

#include <QQuickWindow>

#include <EGL/egl.h>
#include <EGL/eglext.h>

#include <cstdint>
#include <optional>

QT_FORWARD_DECLARE_CLASS(QSGTexture)

namespace WallpiperKde {

constexpr unsigned int kGlTextureExternalOes = 0x8D65;

class EglDmabufImporter {
public:
  struct Import {
    unsigned int texture = 0;
    EGLImageKHR image = EGL_NO_IMAGE_KHR;
    int syncFd = -1;
  };

  bool ensureBound(QQuickWindow *window);
  bool isBound() const;

  std::optional<Import> importDmabuf(int width, int height, uint32_t stride,
                                     uint64_t modifier, int fd) const;
  void destroyImport(const Import &import) const;
  void refreshBinding(const Import &import) const;

  std::optional<EGLImageKHR> createImageOnly(int width, int height,
                                             uint32_t stride, uint64_t modifier,
                                             int fd) const;
  void destroyEglImage(EGLImageKHR image) const;

  // Takes ownership of syncFd: it is always consumed (closed by EGL on success,
  // by us on failure).
  bool waitForSyncFd(int syncFd) const;

  QSGTexture *wrapExternalOes(QQuickWindow *window, const Import &import,
                              const QSize &size) const;
  QSGTexture *wrapTexture2D(QQuickWindow *window, unsigned int texture,
                            const QSize &size) const;

private:
  using GlEglImageTargetTexture2DOesFn = void (*)(unsigned int target,
                                                  void *image);
  using EglCreateSyncKHRFn = EGLSyncKHR (*)(EGLDisplay, EGLenum,
                                            const EGLint *);
  using EglDestroySyncKHRFn = EGLBoolean (*)(EGLDisplay, EGLSyncKHR);
  using EglClientWaitSyncKHRFn = EGLint (*)(EGLDisplay, EGLSyncKHR, EGLint,
                                            EGLTimeKHR);

  EGLDisplay m_display = EGL_NO_DISPLAY;
  GlEglImageTargetTexture2DOesFn m_glEGLImageTargetTexture2DOES = nullptr;
  EglCreateSyncKHRFn m_eglCreateSyncKHR = nullptr;
  EglDestroySyncKHRFn m_eglDestroySyncKHR = nullptr;
  EglClientWaitSyncKHRFn m_eglClientWaitSyncKHR = nullptr;
  bool m_loggedUnsupported = false;
};

} // namespace WallpiperKde
