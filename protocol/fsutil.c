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
