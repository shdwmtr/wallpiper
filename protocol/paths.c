#include "wallpiper/protocol.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

static bool env_path(const char *name, const char **out) {
  const char *v = getenv(name);
  if (v && v[0] != '\0') {
    *out = v;
    return true;
  }
  return false;
}

static bool fmt_ok(char *out, size_t out_len, const char *fmt, ...) {
  va_list args;
  va_start(args, fmt);
  int n = vsnprintf(out, out_len, fmt, args);
  va_end(args);
  return n > 0 && (size_t)n < out_len;
}

bool wp_ctl_socket_path(const char *portal_name, char *out, size_t out_len) {
  if (!portal_name || portal_name[0] == '\0') {
    return false;
  }
  return fmt_ok(out, out_len, "/tmp/wallpiper-portal-%s-ctl.sock", portal_name);
}

bool wp_temp_dir(char *out, size_t out_len) {
  const char *v;
  if (env_path("WALLPIPER_TEMP_DIR", &v)) {
    return fmt_ok(out, out_len, "%s", v);
  }
  return fmt_ok(out, out_len, "/tmp/wallpiper");
}

bool wp_daemon_ctl_socket_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiperd-ctl.sock", temp_dir);
}

bool wp_runtime_dir(char *out, size_t out_len) {
  const char *v;
  if (env_path("WALLPIPER_RUNTIME_DIR", &v)) {
    return fmt_ok(out, out_len, "%s", v);
  }

  const char *state_home;
  if (env_path("XDG_STATE_HOME", &state_home)) {
    return fmt_ok(out, out_len, "%s/wallpiper", state_home);
  }

  const char *home;
  if (!env_path("HOME", &home)) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/.local/state/wallpiper", home);
}

bool wp_windowbrowser_socket_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiper-windowbrowser.sock", temp_dir);
}

bool wp_inject_socket_path(char *out, size_t out_len) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    return false;
  }
  return fmt_ok(out, out_len, "%s/wallpiper-inject.sock", temp_dir);
}
