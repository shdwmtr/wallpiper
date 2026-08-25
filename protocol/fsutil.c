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

#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

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

static bool copy_file_with_mode(const char *src, const char *dst, mode_t mode) {
  int in_fd = open(src, O_RDONLY);
  if (in_fd < 0) {
    return false;
  }
  int out_fd = open(dst, O_WRONLY | O_CREAT | O_TRUNC, mode);
  if (out_fd < 0) {
    close(in_fd);
    return false;
  }

  char buf[65536];
  bool ok = true;
  ssize_t n;
  while ((n = read(in_fd, buf, sizeof(buf))) > 0) {
    ssize_t off = 0;
    while (off < n) {
      ssize_t written = write(out_fd, buf + off, (size_t)(n - off));
      if (written <= 0) {
        ok = false;
        break;
      }
      off += written;
    }
    if (!ok) {
      break;
    }
  }
  if (n < 0) {
    ok = false;
  }

  close(in_fd);
  close(out_fd);
  if (ok) {
    chmod(dst, mode);
  }
  return ok;
}

bool wp_sync_dir_tree(const char *src_dir, const char *dst_dir) {
  DIR *d = opendir(src_dir);
  if (!d) {
    return false;
  }
  if (!wp_mkdir_p(dst_dir)) {
    closedir(d);
    return false;
  }

  bool ok = true;
  struct dirent *entry;
  while ((entry = readdir(d)) != NULL) {
    if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) {
      continue;
    }
    if (strcmp(entry->d_name, "config.json") == 0) {
      continue;
    }

    char src_path[1024];
    char dst_path[1024];
    if (snprintf(src_path, sizeof(src_path), "%s/%s", src_dir,
                 entry->d_name) >= (int)sizeof(src_path) ||
        snprintf(dst_path, sizeof(dst_path), "%s/%s", dst_dir,
                 entry->d_name) >= (int)sizeof(dst_path)) {
      ok = false;
      continue;
    }

    struct stat src_st;
    if (lstat(src_path, &src_st) != 0) {
      ok = false;
      continue;
    }

    if (S_ISDIR(src_st.st_mode)) {
      if (!wp_sync_dir_tree(src_path, dst_path)) {
        ok = false;
      }
      continue;
    }

    if (!S_ISREG(src_st.st_mode)) {
      continue;
    }

    struct stat dst_st;
    if (stat(dst_path, &dst_st) == 0) {
      continue;
    }

    if (!copy_file_with_mode(src_path, dst_path, src_st.st_mode & 0777)) {
      ok = false;
    }
  }

  closedir(d);
  return ok;
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
