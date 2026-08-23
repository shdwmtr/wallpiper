/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2026 Ethan Alexander
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#include <stdio.h>
#include <string.h>

#include "wallpiper/daemon_ctl_protocol.h"
#include "wallpiper/steam_paths.h"

static const char *const DAEMON_COMMANDS[] = {
    "debug-on",
    "debug-off",
};
#define DAEMON_COMMAND_COUNT                                                   \
  (sizeof(DAEMON_COMMANDS) / sizeof(DAEMON_COMMANDS[0]))

static bool is_daemon_command(const char *cmd) {
  for (size_t i = 0; i < DAEMON_COMMAND_COUNT; i++) {
    if (strcmp(cmd, DAEMON_COMMANDS[i]) == 0) {
      return true;
    }
  }
  return false;
}

static void print_usage(void) {
  fprintf(stderr, "usage: wallpiperctl <command>\n"
                  "\n"
                  "daemon commands (require a running wallpiperd):\n"
                  "  debug-on | debug-off\n"
                  "\n"
                  "standalone commands:\n"
                  "  check-config\n");
}

int main(int argc, char **argv) {
  if (argc < 2) {
    print_usage();
    return 1;
  }

  const char *cmd = argv[1];
  char err[512];
  bool ok;

  if (strcmp(cmd, "check-config") == 0) {
    wp_describe();
    ok = true;
  } else if (is_daemon_command(cmd)) {
    ok = wp_send_daemon_command((const char *const *)(argv + 1),
                                (size_t)(argc - 1), err, sizeof(err));
  } else {
    snprintf(err, sizeof(err), "unknown command: %s", cmd);
    ok = false;
  }

  if (!ok) {
    fprintf(stderr, "wallpiperctl: %s\n", err);
    return 1;
  }
  return 0;
}
