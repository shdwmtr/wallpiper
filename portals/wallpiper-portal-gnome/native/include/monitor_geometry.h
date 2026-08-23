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
void wallpiper_monitor_detect_all(MetaBackend *backend,
                                  WallpiperMonitorGeometry *out, guint max,
                                  guint *out_count);

void wallpiper_monitor_detect_all_named(MetaBackend *backend,
                                        WallpiperMonitorGeometry *out,
                                        char *connector_names_out,
                                        size_t connector_name_len, guint max,
                                        guint *out_count);

gboolean wallpiper_x11_output_for_size(guint32 width, guint32 height,
                                       char *name_out, size_t name_out_len);

gchar *
wallpiper_monitor_geometry_to_json(const WallpiperMonitorGeometry *geometry);
G_END_DECLS
