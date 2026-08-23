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

#include "signals.h"

#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>

static void *sigchld_thread_main(void *arg) {
  (void)arg;

  sigset_t set;
  sigemptyset(&set);
  sigaddset(&set, SIGCHLD);

  for (;;) {
    int sig;
    if (sigwait(&set, &sig) != 0) {
      continue;
    }
    for (;;) {
      int status;
      pid_t pid = waitpid(-1, &status, WNOHANG);
      if (pid <= 0) {
        break;
      }
    }
  }

  return NULL;
}

void wp_reap_children_forever(void) {
  sigset_t set;
  sigemptyset(&set);
  sigaddset(&set, SIGCHLD);
  pthread_sigmask(SIG_BLOCK, &set, NULL);

  pthread_t thread;
  if (pthread_create(&thread, NULL, sigchld_thread_main, NULL) != 0) {
    fprintf(stderr, "failed to install SIGCHLD handler\n");
    exit(1);
  }
  pthread_detach(thread);
}

static wp_shutdown_fn g_shutdown_fn = NULL;

static void *shutdown_thread_main(void *arg) {
  (void)arg;

  sigset_t set;
  sigemptyset(&set);
  sigaddset(&set, SIGINT);
  sigaddset(&set, SIGTERM);

  int sig;
  if (sigwait(&set, &sig) == 0) {
    if (g_shutdown_fn) {
      g_shutdown_fn();
    }
    exit(0);
  }
  return NULL;
}

void wp_install_shutdown_handler(wp_shutdown_fn on_shutdown) {
  g_shutdown_fn = on_shutdown;

  sigset_t set;
  sigemptyset(&set);
  sigaddset(&set, SIGINT);
  sigaddset(&set, SIGTERM);
  pthread_sigmask(SIG_BLOCK, &set, NULL);

  pthread_t thread;
  if (pthread_create(&thread, NULL, shutdown_thread_main, NULL) != 0) {
    fprintf(stderr, "failed to install shutdown signal handler\n");
    exit(1);
  }
  pthread_detach(thread);
}

void wp_ignore_sigpipe(void) { signal(SIGPIPE, SIG_IGN); }
