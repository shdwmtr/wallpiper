#include "commands.h"

#include "config.h"
#include "portal.h"
#include "renderer.h"

#include <stdio.h>
#include <string.h>

#include "wallpiper/steam_paths.h"
#include "wallpiper/wallpaper_catalog.h"

static bool set_wallpaper(const char *const *args, size_t arg_count,
                          char *err_out, size_t err_out_len) {
  const char *const *rest;
  size_t rest_count;

  if (arg_count >= 1 && strcmp(args[0], "--id") == 0) {
    if (arg_count < 2) {
      snprintf(err_out, err_out_len,
               "usage: set <file> [location]\n       set --id <workshop_id> "
               "[location]");
      return false;
    }
    const char *id = args[1];
    rest = args + 2;
    rest_count = arg_count - 2;

    char workshop_dir[1024];
    if (!wp_workshop_content_dir(workshop_dir, sizeof(workshop_dir), err_out,
                                 err_out_len)) {
      return false;
    }
    if (!wp_wallpaper_catalog_resolve(workshop_dir, id, err_out, err_out_len)) {
      return false;
    }
  } else if (arg_count == 0) {
    snprintf(err_out, err_out_len,
             "usage: set <file> [location]\n       set --id <workshop_id> "
             "[location]");
    return false;
  } else {
    rest = args + 1;
    rest_count = arg_count - 1;
  }

  const char *location = rest_count > 0 ? rest[0] : "default";

  wp_monitor_geometry_t monitor;
  bool have_monitor = wp_portal_current_monitor(&monitor);
  if (!have_monitor) {
    char portal_name[64];
    char perr[256];
    if (wp_portal_name(portal_name, sizeof(portal_name), perr, sizeof(perr))) {
      have_monitor = wp_portal_query_monitor_once(portal_name, &monitor);
    }
  }
  if (!have_monitor) {
    monitor = WP_FALLBACK_MONITOR;
  }

  wp_renderer_swap(location, monitor);
  return true;
}

static bool trigger_windowbrowser(char *err_out, size_t err_out_len) {
  (void)err_out;
  (void)err_out_len;
  return true;
}

static bool trigger_inject(const char *const *args, size_t arg_count,
                           char *err_out, size_t err_out_len) {
  (void)args;
  (void)arg_count;
  (void)err_out;
  (void)err_out_len;
  return true;
}

bool wp_commands_dispatch(const char *const *args, size_t arg_count,
                          char *err_out, size_t err_out_len) {
  if (arg_count == 0) {
    snprintf(err_out, err_out_len, "empty command");
    return false;
  }

  const char *cmd = args[0];
  const char *const *rest = args + 1;
  size_t rest_count = arg_count - 1;

  if (strcmp(cmd, "pause") == 0) {
    wp_renderer_set_paused(true);
    return true;
  }
  if (strcmp(cmd, "resume") == 0) {
    wp_renderer_set_paused(false);
    return true;
  }
  if (strcmp(cmd, "debug") == 0) {
    wp_portal_set_debug_overlay(true);
    return true;
  }
  if (strcmp(cmd, "nodebug") == 0) {
    wp_portal_set_debug_overlay(false);
    return true;
  }
  if (strcmp(cmd, "set") == 0) {
    return set_wallpaper(rest, rest_count, err_out, err_out_len);
  }
  if (strcmp(cmd, "windowbrowser") == 0) {
    return trigger_windowbrowser(err_out, err_out_len);
  }
  if (strcmp(cmd, "inject") == 0) {
    return trigger_inject(rest, rest_count, err_out, err_out_len);
  }

  snprintf(err_out, err_out_len, "unknown command: %s", cmd);
  return false;
}
