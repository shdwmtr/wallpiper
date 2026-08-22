#pragma once

#include <pthread.h>
#include <stdarg.h>
#include <syslog.h>

#if defined(__GNUC__) || defined(__clang__)
#define WP_PRINTF_LIKE(fmt_idx, arg_idx)                                       \
  __attribute__((format(printf, fmt_idx, arg_idx)))
#else
#define WP_PRINTF_LIKE(fmt_idx, arg_idx)
#endif

#ifndef LOG_IDENTIFIER
#define LOG_IDENTIFIER "wallpiper"
#endif

static pthread_once_t wp_log_once = PTHREAD_ONCE_INIT;

static void wp_log_init(void) { openlog(LOG_IDENTIFIER, LOG_PID, LOG_USER); }

static void wp_log(const char *fmt, ...) WP_PRINTF_LIKE(1, 2);

static inline void wp_log(const char *fmt, ...) {
  pthread_once(&wp_log_once, wp_log_init);

  va_list args;
  va_start(args, fmt);
  vsyslog(LOG_INFO, fmt, args);
  va_end(args);
}
