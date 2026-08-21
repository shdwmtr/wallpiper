#pragma once
#include <glib.h>
#include <meta/meta-backend.h>

G_BEGIN_DECLS
typedef struct {
  int x;
  int y;
  guint32 width;
  guint32 height;
  guint32 logical_width;
  guint32 logical_height;
  gdouble scale;
} WallpiperMonitorGeometry;

WallpiperMonitorGeometry wallpiper_monitor_detect_primary(MetaBackend *backend);
gchar *
wallpiper_monitor_geometry_to_json(const WallpiperMonitorGeometry *geometry);
G_END_DECLS
