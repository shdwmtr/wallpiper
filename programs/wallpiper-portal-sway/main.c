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

#include <cJSON.h>

#include "wallpiper-wl/portal.h"
#include "wallpiper/monitor_geometry.h"

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

static bool try_detect_geometry(wp_monitor_geometry_t *out) {
  const char *argv[] = {"swaymsg", "-t", "get_outputs", "-r", NULL};
  char *output = run_command(argv);
  if (!output) {
    return false;
  }

  cJSON *outputs = cJSON_Parse(output);
  free(output);
  if (!outputs || !cJSON_IsArray(outputs)) {
    cJSON_Delete(outputs);
    return false;
  }

  bool found = false;
  int count = cJSON_GetArraySize(outputs);
  for (int i = 0; i < count; i++) {
    cJSON *item = cJSON_GetArrayItem(outputs, i);
    cJSON *focused = cJSON_GetObjectItem(item, "focused");
    cJSON *active = cJSON_GetObjectItem(item, "active");
    if (!focused || !cJSON_IsTrue(focused) || !active ||
        !cJSON_IsTrue(active)) {
      continue;
    }

    cJSON *rect = cJSON_GetObjectItem(item, "rect");
    cJSON *x = cJSON_GetObjectItem(rect, "x");
    cJSON *y = cJSON_GetObjectItem(rect, "y");
    cJSON *width = cJSON_GetObjectItem(rect, "width");
    cJSON *height = cJSON_GetObjectItem(rect, "height");
    cJSON *scale = cJSON_GetObjectItem(item, "scale");
    if (!cJSON_IsNumber(x) || !cJSON_IsNumber(y) || !cJSON_IsNumber(width) ||
        !cJSON_IsNumber(height)) {
      break;
    }

    double scale_value = cJSON_IsNumber(scale) ? scale->valuedouble : 1.0;
    wp_wl_geometry_from_scale((int32_t)x->valuedouble, (int32_t)y->valuedouble,
                              (uint32_t)width->valuedouble,
                              (uint32_t)height->valuedouble, scale_value, out);
    found = true;
    break;
  }

  cJSON_Delete(outputs);
  return found;
}

int main(void) {
  wp_wl_portal_config_t config = {
      .portal_name = "sway",
      .layer_namespace = "wallpiper-portal-sway",
      .try_geometry = try_detect_geometry,
      .cursor_pos = NULL,
      .cursor_ctx = NULL,
  };
  wp_wl_portal_run(&config);
  return 0;
}
