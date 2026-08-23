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

#include "ipc.h"
#include "pe_iat.h"
#include "util.h"
#include "win32/libc.h"

typedef HANDLE(WINAPI *CreateFileA_t)(LPCSTR, DWORD, DWORD,
                                      LPSECURITY_ATTRIBUTES, DWORD, DWORD,
                                      HANDLE);
typedef HANDLE(WINAPI *CreateFileW_t)(LPCWSTR, DWORD, DWORD,
                                      LPSECURITY_ATTRIBUTES, DWORD, DWORD,
                                      HANDLE);

static CreateFileA_t real_CreateFileA;
static CreateFileW_t real_CreateFileW;

static void maybe_capture_command_client_handle(const char *namebuf,
                                                DWORD dwDesiredAccess,
                                                HANDLE h) {
  if (h == INVALID_HANDLE_VALUE) {
    return;
  }
  if (!ansi_contains_ci(namebuf, "WPEhandlerBrowseWallpapersClient")) {
    return;
  }
  if (!(dwDesiredAccess & GENERIC_WRITE)) {
    return;
  }

  char logbuf[160];
  wsprintfA(
      logbuf,
      "maybe_capture_command_client_handle: captured handle=%p access=0x%lx", h,
      (unsigned long)dwDesiredAccess);
  debug_log(logbuf);
}

static HANDLE WINAPI fake_CreateFileA(
    LPCSTR lpFileName, DWORD dwDesiredAccess, DWORD dwShareMode,
    LPSECURITY_ATTRIBUTES lpSecurityAttributes, DWORD dwCreationDisposition,
    DWORD dwFlagsAndAttributes, HANDLE hTemplateFile) {

  HANDLE h = real_CreateFileA
                 ? real_CreateFileA(lpFileName, dwDesiredAccess, dwShareMode,
                                    lpSecurityAttributes, dwCreationDisposition,
                                    dwFlagsAndAttributes, hTemplateFile)
                 : INVALID_HANDLE_VALUE;

  if (lpFileName && ansi_starts_with_ci(lpFileName, "\\\\.\\pipe\\")) {
    maybe_capture_command_client_handle(lpFileName, dwDesiredAccess, h);
  }
  return h;
}

static HANDLE WINAPI fake_CreateFileW(
    LPCWSTR lpFileName, DWORD dwDesiredAccess, DWORD dwShareMode,
    LPSECURITY_ATTRIBUTES lpSecurityAttributes, DWORD dwCreationDisposition,
    DWORD dwFlagsAndAttributes, HANDLE hTemplateFile) {
  HANDLE h = real_CreateFileW
                 ? real_CreateFileW(lpFileName, dwDesiredAccess, dwShareMode,
                                    lpSecurityAttributes, dwCreationDisposition,
                                    dwFlagsAndAttributes, hTemplateFile)
                 : INVALID_HANDLE_VALUE;

  if (lpFileName && wide_starts_with_ci(lpFileName, L"\\\\.\\pipe\\")) {
    char namebuf[MAX_PATH];
    int n = WideCharToMultiByte(CP_ACP, 0, lpFileName, -1, namebuf,
                                sizeof(namebuf), NULL, NULL);
    if (n > 0) {
      maybe_capture_command_client_handle(namebuf, dwDesiredAccess, h);
    }
  }
  return h;
}

void install_ipc_file_hooks_and_poll(void) {
  real_CreateFileA = (CreateFileA_t)(void *)patch_iat(
      GetModuleHandleW(NULL), "KERNEL32.dll", "CreateFileA",
      (FARPROC)fake_CreateFileA);
  real_CreateFileW = (CreateFileW_t)(void *)patch_iat(
      GetModuleHandleW(NULL), "KERNEL32.dll", "CreateFileW",
      (FARPROC)fake_CreateFileW);
}
