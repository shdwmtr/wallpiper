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

#include "wallpiper/fsutil.h"

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

bool wp_mkdir_p(const char *path) {
  char buf[1024];
  int n = snprintf(buf, sizeof(buf), "%s", path);
  if (n <= 0 || (size_t)n >= sizeof(buf)) {
    return false;
  }

  for (char *p = buf + 1; *p; p++) {
    if (*p != '/') {
      continue;
    }
    *p = '\0';
    if (mkdir(buf, 0755) != 0 && errno != EEXIST) {
      return false;
    }
    *p = '/';
  }

  if (mkdir(buf, 0755) != 0 && errno != EEXIST) {
    return false;
  }
  return true;
}

bool wp_mkdir_p_parent(const char *file_path) {
  char buf[1024];
  int n = snprintf(buf, sizeof(buf), "%s", file_path);
  if (n <= 0 || (size_t)n >= sizeof(buf)) {
    return false;
  }

  char *slash = strrchr(buf, '/');
  if (!slash || slash == buf) {
    return true;
  }
  *slash = '\0';
  return wp_mkdir_p(buf);
}
