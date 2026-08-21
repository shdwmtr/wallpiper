#include "monitor_geometry.h"
#include "mutter_private.h"

WallpiperMonitorGeometry
wallpiper_monitor_detect_primary(MetaBackend *backend) {
  WallpiperMonitorGeometry fallback = {0, 0, 1920, 1080, 1920, 1080, 1.0};

  MetaMonitorManager *manager = meta_backend_get_monitor_manager(backend);
  if (!manager)
    return fallback;

  MetaLogicalMonitor *logical_monitor =
      meta_monitor_manager_get_primary_logical_monitor(manager);
  if (!logical_monitor) {
    GList *logical_monitors =
        meta_monitor_manager_get_logical_monitors(manager);
    if (!logical_monitors)
      return fallback;
    logical_monitor = logical_monitors->data;
  }

  MtkRectangle layout = meta_logical_monitor_get_layout(logical_monitor);
  gdouble scale = (gdouble)meta_logical_monitor_get_scale(logical_monitor);
  if (scale <= 0.0) {
    scale = 1.0;
  }

  WallpiperMonitorGeometry geometry = {
      .x = layout.x,
      .y = layout.y,
      .width = (guint32)layout.width,
      .height = (guint32)layout.height,
      .logical_width = (guint32)layout.width,
      .logical_height = (guint32)layout.height,
      .scale = scale,
  };

  GList *monitors = meta_logical_monitor_get_monitors(logical_monitor);
  if (monitors) {
    MetaMonitor *monitor = monitors->data;
    MetaMonitorMode *mode = meta_monitor_get_current_mode(monitor);
    if (mode) {
      int width = 0, height = 0;
      meta_monitor_mode_get_resolution(mode, &width, &height);
      if (width > 0 && height > 0) {
        geometry.width = (guint32)width;
        geometry.height = (guint32)height;
      }
    }
  }

  return geometry;
}

gchar *wallpiper_monitor_geometry_to_json(const WallpiperMonitorGeometry *g) {
  return g_strdup_printf(
      "{\"x\":%d,\"y\":%d,\"width\":%u,\"height\":%u,"
      "\"logical_width\":%u,\"logical_height\":%u,\"scale\":%g}",
      g->x, g->y, g->width, g->height, g->logical_width, g->logical_height,
      g->scale);
}
