#pragma once

#include <stddef.h>

typedef struct {
  int pids[256];
  size_t count;
} wp_pid_list_t;

void wp_kill_pids_gracefully(const int *pids, size_t count);

void wp_find_renderer_pids(wp_pid_list_t *out);
void wp_find_proton_wrapper_pids(wp_pid_list_t *out);
void wp_find_picker_pids(wp_pid_list_t *out);
void wp_find_webwallpaper_pids(wp_pid_list_t *out);
