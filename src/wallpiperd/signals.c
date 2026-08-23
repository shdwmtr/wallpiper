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
