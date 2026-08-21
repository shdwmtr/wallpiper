#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "wallpiper/monitor_geometry.h"

typedef enum {
  WP_PORTAL_SPAWN,
  WP_PORTAL_EXTERNALLY_MANAGED,
} wp_portal_strategy_kind_t;

typedef struct {
  wp_portal_strategy_kind_t kind;
  char name[64];
  char binary[1024];
} wp_portal_strategy_t;

void wp_portal_spawn_strategy(const char *name, wp_portal_strategy_t *out);
void wp_portal_spawn(const wp_portal_strategy_t *strategy);
void wp_portal_spawn_geometry_watcher(const char *name, bool patient);
bool wp_portal_wait_for_geometry(wp_monitor_geometry_t *out);
bool wp_portal_current_monitor(wp_monitor_geometry_t *out);
bool wp_portal_query_monitor_once(const char *name, wp_monitor_geometry_t *out);
bool wp_portal_detach_display(void);
void wp_portal_set_debug_overlay(bool enabled);
bool wp_portal_take_display_pid(int *out_pid);
