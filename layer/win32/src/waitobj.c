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

#include "waitobj.h"
#include "pe_iat.h"
#include "util.h"
#include "win32/imports.h"
#include "win32/libc.h"

typedef struct wp_wait_object_s {
  HANDLE target;
  WAITORTIMERCALLBACK callback;
  PVOID context;
  DWORD timeout_ms;
  ULONG flags;
  volatile LONG cancelled;
} wp_wait_object_t;

static DWORD WINAPI wait_thread_proc(LPVOID param) {
  wp_wait_object_t *w = (wp_wait_object_t *)param;
  DWORD elapsed = 0;
  char logbuf[128];

  for (;;) {
    if (InterlockedCompareExchange(&w->cancelled, 0, 0)) {
      debug_log("waiter_thread_proc: cancelled, exiting");
      break;
    }

    DWORD slice = 200;
    if (w->timeout_ms != INFINITE) {
      DWORD left = (w->timeout_ms > elapsed) ? (w->timeout_ms - elapsed) : 0;
      if (left < slice)
        slice = left;
    }

    DWORD result = WaitForSingleObject(w->target, slice);
    elapsed += slice;

    wsprintfA(logbuf, "waiter_thread_proc: slice=%lu result=%lu elapsed=%lu",
              (unsigned long)slice, (unsigned long)result,
              (unsigned long)elapsed);
    debug_log(logbuf);

    DWORD exit_code = 0;
    BOOL got_exit_code = GetExitCodeProcess(w->target, &exit_code);
    wsprintfA(logbuf, "waiter_thread_proc: GetExitCodeProcess=%d exitCode=%lu",
              got_exit_code, (unsigned long)exit_code);
    debug_log(logbuf);

    if (result == WAIT_OBJECT_0) {
      debug_log("waiter_thread_proc: calling callback (signaled)");
      w->callback(w->context, FALSE);
      debug_log("waiter_thread_proc: callback returned (signaled)");
      break;
    }

    if (w->timeout_ms != INFINITE && elapsed >= w->timeout_ms) {
      debug_log("waiter_thread_proc: calling callback (timeout)");
      w->callback(w->context, TRUE);
      debug_log("waiter_thread_proc: callback returned (timeout)");
      if (w->flags & WT_EXECUTEONLYONCE) {
        break;
      }
      elapsed = 0;
    }
  }

  debug_log("waiter_thread_proc: exiting");
  free(w);
  return 0;
}

static BOOL WINAPI fake_RegisterWaitForSingleObject(
    PHANDLE phNewWaitObject, HANDLE hObject, WAITORTIMERCALLBACK Callback,
    PVOID Context, ULONG dwMilliseconds, ULONG dwFlags) {
  if (!phNewWaitObject || !Callback) {
    return FALSE;
  }

  wp_wait_object_t *w = (wp_wait_object_t *)malloc(sizeof(wp_wait_object_t));
  if (!w) {
    return FALSE;
  }
  w->target = hObject;
  w->callback = Callback;
  w->context = Context;
  w->timeout_ms = dwMilliseconds;
  w->flags = dwFlags;
  w->cancelled = 0;

  HANDLE thread = CreateThread(NULL, 0, wait_thread_proc, w, 0, NULL);
  if (!thread) {
    free(w);
    return FALSE;
  }
  CloseHandle(thread);

  *phNewWaitObject = (HANDLE)w;
  debug_log("fake_RegisterWaitForSingleObject: registered");
  return TRUE;
}

static BOOL WINAPI fake_UnregisterWait(HANDLE WaitHandle) {
  if (!WaitHandle) {
    return FALSE;
  }
  wp_wait_object_t *w = (wp_wait_object_t *)WaitHandle;
  InterlockedExchange(&w->cancelled, 1);
  debug_log("fake_UnregisterWait: cancelled");
  return TRUE;
}

static BOOL WINAPI fake_UnregisterWaitEx(HANDLE WaitHandle,
                                         HANDLE CompletionEvent) {
  (void)CompletionEvent;
  return fake_UnregisterWait(WaitHandle);
}

void install_wait_hooks(void) {
  FARPROC orig_register = patch_iat(GetModuleHandleW(NULL), "KERNEL32.dll",
                                    "RegisterWaitForSingleObject",
                                    (FARPROC)fake_RegisterWaitForSingleObject);
  debug_log(orig_register
                ? "install_wait_hooks: patch_iat RegisterWaitForSingleObject OK"
                : "install_wait_hooks: patch_iat RegisterWaitForSingleObject "
                  "NOT FOUND");

  FARPROC orig_unregister =
      patch_iat(GetModuleHandleW(NULL), "KERNEL32.dll", "UnregisterWait",
                (FARPROC)fake_UnregisterWait);
  debug_log(orig_unregister
                ? "install_wait_hooks: patch_iat UnregisterWait OK"
                : "install_wait_hooks: patch_iat UnregisterWait NOT FOUND");

  FARPROC orig_unregister_ex =
      patch_iat(GetModuleHandleW(NULL), "KERNEL32.dll", "UnregisterWaitEx",
                (FARPROC)fake_UnregisterWaitEx);
  debug_log(orig_unregister_ex
                ? "install_wait_hooks: patch_iat UnregisterWaitEx OK"
                : "install_wait_hooks: patch_iat UnregisterWaitEx NOT FOUND");
}
