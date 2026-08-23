#include "commands.h"

#include "portal.h"

#include <stdio.h>
#include <string.h>

bool wp_commands_dispatch(const char *const *args, size_t arg_count,
                          char *err_out, size_t err_out_len) {
  if (arg_count == 0) {
    snprintf(err_out, err_out_len, "empty command");
    return false;
  }

  const char *cmd = args[0];

  if (strcmp(cmd, "debug-on") == 0) {
    wp_portal_set_debug_overlay(true);
    return true;
  }
  if (strcmp(cmd, "debug-off") == 0) {
    wp_portal_set_debug_overlay(false);
    return true;
  }

  snprintf(err_out, err_out_len, "unknown command: %s", cmd);
  return false;
}
