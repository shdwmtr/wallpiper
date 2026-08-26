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

#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdint.h>
#include <stdlib.h>
#include <sys/poll.h>
#include <sys/syscall.h>
#include <unistd.h>

#include "log.h"

static int wp_pidfd_open(pid_t pid) {
  return (int)syscall(SYS_pidfd_open, pid, 0);
}

static void *wp_guardian_watch(void *arg) {
  int pidfd = (int)(intptr_t)arg;
  struct pollfd pfd = {.fd = pidfd, .events = POLLIN};

  int n;
  do {
    n = poll(&pfd, 1, -1);
  } while (n < 0 && errno == EINTR);

  wp_log("guardian process gone, self-terminating pid=%d", (int)getpid());
  raise(SIGKILL);
  return NULL;
}

__attribute__((constructor)) static void wp_register_guardian(void) {
  const char *guardian_str = getenv("WALLPIPER_GUARDIAN_PID");
  if (!guardian_str || !*guardian_str) {
    return;
  }

  pid_t guardian_pid = (pid_t)strtol(guardian_str, NULL, 10);
  if (guardian_pid <= 0) {
    wp_log("invalid WALLPIPER_GUARDIAN_PID=%s", guardian_str);
    return;
  }

  int pidfd = wp_pidfd_open(guardian_pid);
  if (pidfd < 0) {
    wp_log("guardian pid %d already gone, self-terminating pid=%d",
           (int)guardian_pid, (int)getpid());
    raise(SIGKILL);
    return;
  }

  pthread_t thread;
  if (pthread_create(&thread, NULL, wp_guardian_watch,
                     (void *)(intptr_t)pidfd) != 0) {
    wp_log("failed to start guardian watch thread for pid=%d", (int)getpid());
    close(pidfd);
    return;
  }
  pthread_detach(thread);

  wp_log("watching guardian pid=%d from pid=%d", (int)guardian_pid,
         (int)getpid());
}
