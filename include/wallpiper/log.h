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
