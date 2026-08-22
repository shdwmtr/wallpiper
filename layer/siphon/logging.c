#include "logging.h"

#include "config.h"

#include <pthread.h>
#include <stdarg.h>
#include <stdio.h>
#include <sys/time.h>
#include <unistd.h>

static pthread_mutex_t g_log_mutex = PTHREAD_MUTEX_INITIALIZER;
static FILE *g_log_file = NULL;

void wp_capture_log(const char *fmt, ...) {
  pthread_mutex_lock(&g_log_mutex);

  if (!g_log_file) {
    g_log_file = fopen(WP_CAPTURE_LOG_PATH, "a");
  }
  if (!g_log_file) {
    pthread_mutex_unlock(&g_log_mutex);
    return;
  }

  struct timeval tv;
  gettimeofday(&tv, NULL);
  long long now_ms = (long long)tv.tv_sec * 1000 + tv.tv_usec / 1000;

  fprintf(g_log_file, "[%lld pid=%d] ", now_ms, (int)getpid());
  va_list args;
  va_start(args, fmt);
  vfprintf(g_log_file, fmt, args);
  va_end(args);
  fprintf(g_log_file, "\n");

  if (fflush(g_log_file) != 0) {
    fclose(g_log_file);
    g_log_file = NULL;
  }

  pthread_mutex_unlock(&g_log_mutex);
}

bool wp_capture_should_sample(uint64_t count) {
  return count <= WP_LOG_SAMPLE_WARMUP || count % WP_LOG_SAMPLE_INTERVAL == 0;
}
