#pragma once
#include "monitor_geometry.h"
#include "protocol.h"

#include <EGL/egl.h>
#include <meta/display.h>
#include <meta/meta-backend.h>

G_BEGIN_DECLS

typedef struct {
  gboolean used;
  guint32 width;
  guint32 height;
  CoglTexture *texture;
} WallpiperCaptureSlot;

typedef struct _WallpiperPortalState {
  MetaBackend *backend;
  CoglContext *cogl_context;
  EGLDisplay egl_display;

  ClutterActor *parent;
  ClutterActor *display_actor;

  WallpiperCaptureSlot slots[MAX_CAPTURE_SLOTS];
  WallpiperMonitorGeometry geometry;
  gboolean debug_enabled;

  int capture_socket_fd;
  guint capture_source_id;

  int ctl_socket_fd;
  guint ctl_source_id;

  MetaDisplay *meta_display;
  gulong restacked_handler_id;
  gulong window_created_handler_id;
  gulong monitors_changed_handler_id;
} WallpiperPortalState;

G_END_DECLS
