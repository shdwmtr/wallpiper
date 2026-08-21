#pragma once
#include <meta/meta-backend.h>
#include <meta/meta-logical-monitor.h>
#include <meta/meta-monitor-manager.h>
#include <meta/meta-monitor.h>
#include <mtk/mtk.h>

G_BEGIN_DECLS
extern ClutterBackend *meta_backend_get_clutter_backend(MetaBackend *backend);
typedef struct _MetaMonitorMode MetaMonitorMode;

extern MetaLogicalMonitor *
meta_monitor_manager_get_primary_logical_monitor(MetaMonitorManager *manager);
extern MtkRectangle
meta_logical_monitor_get_layout(MetaLogicalMonitor *logical_monitor);
extern float
meta_logical_monitor_get_scale(MetaLogicalMonitor *logical_monitor);
extern MetaMonitorMode *meta_monitor_get_current_mode(MetaMonitor *monitor);
extern void meta_monitor_mode_get_resolution(MetaMonitorMode *mode, int *width,
                                             int *height);
G_END_DECLS
