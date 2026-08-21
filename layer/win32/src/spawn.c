#include "spawn.h"
#include "pe_iat.h"
#include "util.h"
#include "win32/imports.h"
#include "win32/libc.h"

typedef BOOL(WINAPI *CreateProcessA_t)(LPCSTR, LPSTR, LPSECURITY_ATTRIBUTES,
                                       LPSECURITY_ATTRIBUTES, BOOL, DWORD,
                                       LPVOID, LPCSTR, LPSTARTUPINFOA,
                                       LPPROCESS_INFORMATION);
typedef BOOL(WINAPI *CreateProcessW_t)(LPCWSTR, LPWSTR, LPSECURITY_ATTRIBUTES,
                                       LPSECURITY_ATTRIBUTES, BOOL, DWORD,
                                       LPVOID, LPCWSTR, LPSTARTUPINFOW,
                                       LPPROCESS_INFORMATION);

static CreateProcessA_t real_CreateProcessA;
static CreateProcessW_t real_CreateProcessW;

static BOOL WINAPI fake_CreateProcessA(
    LPCSTR lpApplicationName, LPSTR lpCommandLine,
    LPSECURITY_ATTRIBUTES lpProcessAttributes,
    LPSECURITY_ATTRIBUTES lpThreadAttributes, BOOL bInheritHandles,
    DWORD dwCreationFlags, LPVOID lpEnvironment, LPCSTR lpCurrentDirectory,
    LPSTARTUPINFOA lpStartupInfo, LPPROCESS_INFORMATION lpProcessInformation) {
  BOOL ok = real_CreateProcessA
                ? real_CreateProcessA(lpApplicationName, lpCommandLine,
                                      lpProcessAttributes, lpThreadAttributes,
                                      bInheritHandles, dwCreationFlags,
                                      lpEnvironment, lpCurrentDirectory,
                                      lpStartupInfo, lpProcessInformation)
                : FALSE;
  DWORD err = ok ? 0 : GetLastError();

  char logbuf[768];
  wsprintfA(
      logbuf,
      "fake_CreateProcessA: app=%s cmdline=%s cwd=%s -> ok=%d err=%lu pid=%lu",
      lpApplicationName ? lpApplicationName : "(null)",
      lpCommandLine ? lpCommandLine : "(null)",
      lpCurrentDirectory ? lpCurrentDirectory : "(null)", ok,
      (unsigned long)err,
      ok ? (unsigned long)lpProcessInformation->dwProcessId : 0);
  spawn_dump_log(logbuf);

  return ok;
}

static BOOL WINAPI fake_CreateProcessW(
    LPCWSTR lpApplicationName, LPWSTR lpCommandLine,
    LPSECURITY_ATTRIBUTES lpProcessAttributes,
    LPSECURITY_ATTRIBUTES lpThreadAttributes, BOOL bInheritHandles,
    DWORD dwCreationFlags, LPVOID lpEnvironment, LPCWSTR lpCurrentDirectory,
    LPSTARTUPINFOW lpStartupInfo, LPPROCESS_INFORMATION lpProcessInformation) {
  BOOL ok = real_CreateProcessW
                ? real_CreateProcessW(lpApplicationName, lpCommandLine,
                                      lpProcessAttributes, lpThreadAttributes,
                                      bInheritHandles, dwCreationFlags,
                                      lpEnvironment, lpCurrentDirectory,
                                      lpStartupInfo, lpProcessInformation)
                : FALSE;
  DWORD err = ok ? 0 : GetLastError();

  char app_buf[MAX_PATH];
  char cmd_buf[512];
  char cwd_buf[MAX_PATH];
  app_buf[0] = 0;
  cmd_buf[0] = 0;
  cwd_buf[0] = 0;
  if (lpApplicationName)
    WideCharToMultiByte(CP_ACP, 0, lpApplicationName, -1, app_buf,
                        sizeof(app_buf), NULL, NULL);
  if (lpCommandLine)
    WideCharToMultiByte(CP_ACP, 0, lpCommandLine, -1, cmd_buf, sizeof(cmd_buf),
                        NULL, NULL);
  if (lpCurrentDirectory)
    WideCharToMultiByte(CP_ACP, 0, lpCurrentDirectory, -1, cwd_buf,
                        sizeof(cwd_buf), NULL, NULL);

  char logbuf[768];
  wsprintfA(
      logbuf,
      "fake_CreateProcessW: app=%s cmdline=%s cwd=%s -> ok=%d err=%lu pid=%lu",
      app_buf[0] ? app_buf : "(null)", cmd_buf[0] ? cmd_buf : "(null)",
      cwd_buf[0] ? cwd_buf : "(null)", ok, (unsigned long)err,
      ok ? (unsigned long)lpProcessInformation->dwProcessId : 0);
  spawn_dump_log(logbuf);

  return ok;
}

void install_spawn_hooks(void) {
  real_CreateProcessA = (CreateProcessA_t)(void *)patch_iat(
      GetModuleHandleW(NULL), "KERNEL32.dll", "CreateProcessA",
      (FARPROC)fake_CreateProcessA);
  real_CreateProcessW = (CreateProcessW_t)(void *)patch_iat(
      GetModuleHandleW(NULL), "KERNEL32.dll", "CreateProcessW",
      (FARPROC)fake_CreateProcessW);

  debug_log(real_CreateProcessA
                ? "install_spawn_hooks: patch_iat CreateProcessA OK"
                : "install_spawn_hooks: patch_iat CreateProcessA NOT FOUND");
  debug_log(real_CreateProcessW
                ? "install_spawn_hooks: patch_iat CreateProcessW OK"
                : "install_spawn_hooks: patch_iat CreateProcessW NOT FOUND");
}
