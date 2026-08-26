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

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#include "wallpiper-wl/portal.h"
#include "wallpiper/monitor_geometry.h"

typedef struct {
  bool enabled;
  bool primary;
  bool has_position;
  int32_t x;
  int32_t y;
  bool has_scale;
  double scale;
  bool has_mode;
  uint32_t width;
  uint32_t height;
} cosmic_output_t;

static char *run_command(const char *const argv[]) {
  int pipefd[2];
  if (pipe(pipefd) != 0) {
    return NULL;
  }

  pid_t pid = fork();
  if (pid < 0) {
    close(pipefd[0]);
    close(pipefd[1]);
    return NULL;
  }

  if (pid == 0) {
    close(pipefd[0]);
    dup2(pipefd[1], STDOUT_FILENO);
    close(pipefd[1]);
    execvp(argv[0], (char *const *)argv);
    _exit(127);
  }

  close(pipefd[1]);

  size_t cap = 65536;
  size_t len = 0;
  char *buf = malloc(cap);
  if (!buf) {
    close(pipefd[0]);
    waitpid(pid, NULL, 0);
    return NULL;
  }

  for (;;) {
    if (len + 4096 > cap) {
      cap *= 2;
      char *grown = realloc(buf, cap);
      if (!grown) {
        free(buf);
        close(pipefd[0]);
        waitpid(pid, NULL, 0);
        return NULL;
      }
      buf = grown;
    }
    ssize_t n = read(pipefd[0], buf + len, 4096);
    if (n <= 0) {
      break;
    }
    len += (size_t)n;
  }
  buf[len] = '\0';
  close(pipefd[0]);

  int status = 0;
  waitpid(pid, &status, 0);
  if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
    free(buf);
    return NULL;
  }

  return buf;
}

static const char *skip_ws(const char *s) {
  while (*s == ' ' || *s == '\t') {
    s++;
  }
  return s;
}

static bool line_is(const char *line, const char *prefix) {
  return strncmp(line, prefix, strlen(prefix)) == 0;
}

static void consider_candidate(const cosmic_output_t *cur,
                               cosmic_output_t *best_primary,
                               bool *have_primary,
                               cosmic_output_t *best_fallback,
                               bool *have_fallback) {
  if (!cur->enabled || !cur->has_position || !cur->has_scale ||
      !cur->has_mode) {
    return;
  }
  if (cur->primary && !*have_primary) {
    *best_primary = *cur;
    *have_primary = true;
  }
  if (!*have_fallback) {
    *best_fallback = *cur;
    *have_fallback = true;
  }
}

/*
 * `cosmic-randr` outputs KDL instead of JSON. The block shape it
 * emits is fixed, so this is a small line-oriented scan rather than a
 * general KDL parser. It might be entirely chopped, we will have to see.
 */
static bool try_detect_geometry(wp_monitor_geometry_t *out) {
  const char *argv[] = {"cosmic-randr", "list", "--kdl", NULL};
  char *output = run_command(argv);
  if (!output) {
    return false;
  }

  cosmic_output_t cur = {0};
  cosmic_output_t best_primary = {0};
  cosmic_output_t best_fallback = {0};
  bool have_primary = false;
  bool have_fallback = false;

  enum { STATE_TOP, STATE_OUTPUT, STATE_MODES } state = STATE_TOP;

  char *saveptr = NULL;
  char *line = strtok_r(output, "\n", &saveptr);
  while (line) {
    const char *l = skip_ws(line);

    switch (state) {
    case STATE_TOP:
      if (line_is(l, "output ")) {
        cur = (cosmic_output_t){0};
        cur.enabled = strstr(l, "enabled=#true") != NULL;
        state = STATE_OUTPUT;
      }
      break;

    case STATE_OUTPUT:
      if (line_is(l, "position ")) {
        if (sscanf(l, "position %d %d", &cur.x, &cur.y) == 2) {
          cur.has_position = true;
        }
      } else if (line_is(l, "scale ")) {
        if (sscanf(l, "scale %lf", &cur.scale) == 1) {
          cur.has_scale = true;
        }
      } else if (line_is(l, "xwayland_primary")) {
        cur.primary = strstr(l, "#true") != NULL;
      } else if (line_is(l, "modes")) {
        state = STATE_MODES;
      } else if (l[0] == '}') {
        consider_candidate(&cur, &best_primary, &have_primary, &best_fallback,
                           &have_fallback);
        state = STATE_TOP;
      }
      break;

    case STATE_MODES:
      if (l[0] == '}') {
        state = STATE_OUTPUT;
      } else if (line_is(l, "mode ") && strstr(l, "current=#true")) {
        unsigned width, height;
        if (sscanf(l, "mode %u %u", &width, &height) == 2) {
          cur.width = width;
          cur.height = height;
          cur.has_mode = true;
        }
      }
      break;
    }

    line = strtok_r(NULL, "\n", &saveptr);
  }

  free(output);

  const cosmic_output_t *chosen = NULL;
  if (have_primary) {
    chosen = &best_primary;
  } else if (have_fallback) {
    chosen = &best_fallback;
  }
  if (!chosen) {
    return false;
  }

  wp_wl_geometry_from_scale(chosen->x, chosen->y, chosen->width, chosen->height,
                            chosen->scale, out);
  return true;
}

int main(void) {
  wp_wl_portal_config_t config = {
      .portal_name = "cosmic",
      .layer_namespace = "wallpiper-portal-cosmic",
      .try_geometry = try_detect_geometry,
      .cursor_pos = NULL,
      .cursor_ctx = NULL,
  };
  wp_wl_portal_run(&config);
  return 0;
}
