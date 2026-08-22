#pragma once

#include <stdbool.h>

#include "wallpiper/monitor_geometry.h"

void wp_renderer_set_paused(bool paused);
void wp_renderer_spawn(const char *location, wp_monitor_geometry_t monitor);
void wp_renderer_swap(const char *location, wp_monitor_geometry_t monitor);
