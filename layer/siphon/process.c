#include "process.h"

#include <pthread.h>
#include <stdio.h>
#include <string.h>

static bool detect_target_process(void) {
  FILE *f = fopen("/proc/self/comm", "r");
  if (f) {
    char comm[256];
    size_t n = fread(comm, 1, sizeof(comm) - 1, f);
    fclose(f);
    while (n > 0 && (comm[n - 1] == '\n' || comm[n - 1] == '\r')) {
      n--;
    }
    comm[n] = '\0';
    if (strcmp(comm, "wallpaper64.exe") == 0) {
      return true;
    }
  }

  f = fopen("/proc/self/cmdline", "r");
  if (f) {
    char cmdline[8192];
    size_t n = fread(cmdline, 1, sizeof(cmdline) - 1, f);
    fclose(f);
    cmdline[n] = '\0';
    if (memmem(cmdline, n, "webwallpaper64.exe",
               strlen("webwallpaper64.exe")) != NULL) {
      return true;
    }
  }

  return false;
}

static pthread_once_t g_once = PTHREAD_ONCE_INIT;
static bool g_is_target = false;

static void init_once(void) { g_is_target = detect_target_process(); }

bool wp_capture_is_target_process(void) {
  pthread_once(&g_once, init_once);
  return g_is_target;
}
