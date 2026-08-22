#include "util.h"
#include "constants.h"
#include "imports.h"

UINT64 unix_millis(void) {
  FILETIME ft;
  GetSystemTimeAsFileTime(&ft);
  ULARGE_INTEGER uli;
  uli.LowPart = ft.dwLowDateTime;
  uli.HighPart = ft.dwHighDateTime;
  UINT64 t = uli.QuadPart;
  t -= 116444736000000000ULL;
  return t / 10000ULL;
}

void debug_log(const char *msg) {
  HANDLE h = CreateFileA("C:\\dwmapi_shim_debug.log", FILE_APPEND_DATA,
                         FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_ALWAYS,
                         FILE_ATTRIBUTE_NORMAL, NULL);
  if (h == INVALID_HANDLE_VALUE)
    return;
  char prefix[32];
  UINT64 ms = unix_millis();
  wsprintfA(prefix, "[%lu%09lu] ", (unsigned long)(ms / 1000000000ULL),
            (unsigned long)(ms % 1000000000ULL));
  DWORD written;
  WriteFile(h, prefix, (DWORD)lstrlenA(prefix), &written, NULL);
  WriteFile(h, msg, (DWORD)lstrlenA(msg), &written, NULL);
  WriteFile(h, "\r\n", 2, &written, NULL);
  CloseHandle(h);
}

void ipc_dump_log(const char *msg) {
  HANDLE h = CreateFileA("C:\\wallpiper_ipc_dump.log", FILE_APPEND_DATA,
                         FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_ALWAYS,
                         FILE_ATTRIBUTE_NORMAL, NULL);
  if (h == INVALID_HANDLE_VALUE)
    return;
  DWORD written;
  WriteFile(h, msg, (DWORD)lstrlenA(msg), &written, NULL);
  WriteFile(h, "\r\n", 2, &written, NULL);
  CloseHandle(h);
}

void window_dump_log(const char *msg) {
  HANDLE h = CreateFileA("C:\\wallpiper_window_dump.log", FILE_APPEND_DATA,
                         FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_ALWAYS,
                         FILE_ATTRIBUTE_NORMAL, NULL);
  if (h == INVALID_HANDLE_VALUE)
    return;
  DWORD written;
  WriteFile(h, msg, (DWORD)lstrlenA(msg), &written, NULL);
  WriteFile(h, "\r\n", 2, &written, NULL);
  CloseHandle(h);
}

void menu_build_log(const char *msg) {
  HANDLE h = CreateFileA("C:\\wallpiper_menu_build.log", FILE_APPEND_DATA,
                         FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_ALWAYS,
                         FILE_ATTRIBUTE_NORMAL, NULL);
  if (h == INVALID_HANDLE_VALUE)
    return;
  char prefix[32];
  UINT64 ms = unix_millis();
  wsprintfA(prefix, "[%lu%09lu] ", (unsigned long)(ms / 1000000000ULL),
            (unsigned long)(ms % 1000000000ULL));
  DWORD written;
  WriteFile(h, prefix, (DWORD)lstrlenA(prefix), &written, NULL);
  WriteFile(h, msg, (DWORD)lstrlenA(msg), &written, NULL);
  WriteFile(h, "\r\n", 2, &written, NULL);
  CloseHandle(h);
}

BOOL wide_contains_ci_len(const WCHAR *haystack, size_t hlen,
                          const WCHAR *needle) {
  size_t nlen = (size_t)lstrlenW(needle);
  if (nlen == 0 || nlen > hlen)
    return FALSE;
  for (size_t i = 0; i + nlen <= hlen; i++) {
    size_t j = 0;
    for (; j < nlen; j++) {
      WCHAR a = haystack[i + j];
      WCHAR b = needle[j];
      if (a >= L'a' && a <= L'z')
        a = (WCHAR)(a - 32);
      if (b >= L'a' && b <= L'z')
        b = (WCHAR)(b - 32);
      if (a != b)
        break;
    }
    if (j == nlen)
      return TRUE;
  }
  return FALSE;
}

BOOL wide_contains_ci(const WCHAR *haystack, const WCHAR *needle) {
  return wide_contains_ci_len(haystack, (size_t)lstrlenW(haystack), needle);
}

BOOL ansi_contains_ci(const char *haystack, const char *needle) {
  size_t hlen = (size_t)lstrlenA(haystack);
  size_t nlen = (size_t)lstrlenA(needle);
  if (nlen == 0 || nlen > hlen)
    return FALSE;
  for (size_t i = 0; i + nlen <= hlen; i++) {
    size_t j = 0;
    for (; j < nlen; j++) {
      char a = haystack[i + j];
      char b = needle[j];
      if (a >= 'a' && a <= 'z')
        a = (char)(a - 32);
      if (b >= 'a' && b <= 'z')
        b = (char)(b - 32);
      if (a != b)
        break;
    }
    if (j == nlen)
      return TRUE;
  }
  return FALSE;
}

BOOL ansi_starts_with_ci(const char *s, const char *prefix) {
  for (size_t i = 0; prefix[i]; i++) {
    char a = s[i];
    char b = prefix[i];
    if (!a)
      return FALSE;
    if (a >= 'a' && a <= 'z')
      a = (char)(a - 32);
    if (b >= 'a' && b <= 'z')
      b = (char)(b - 32);
    if (a != b)
      return FALSE;
  }
  return TRUE;
}

BOOL wide_starts_with_ci(const WCHAR *s, const WCHAR *prefix) {
  for (size_t i = 0; prefix[i]; i++) {
    WCHAR a = s[i];
    WCHAR b = prefix[i];
    if (!a)
      return FALSE;
    if (a >= L'a' && a <= L'z')
      a = (WCHAR)(a - 32);
    if (b >= L'a' && b <= L'z')
      b = (WCHAR)(b - 32);
    if (a != b)
      return FALSE;
  }
  return TRUE;
}

void basename_w(const WCHAR *path, WCHAR *out) {
  const WCHAR *sep = NULL;
  for (const WCHAR *p = path; *p; p++) {
    if (*p == L'\\' || *p == L'/')
      sep = p;
  }
  lstrcpyW(out, sep ? sep + 1 : path);
}

BOOL get_env_path(const WCHAR *name, WCHAR *out, DWORD out_len) {
  DWORD len = GetEnvironmentVariableW(name, out, out_len);
  return len > 0 && len < out_len;
}

void narrow_maybe_atom(LPCWSTR s, char *out, size_t out_cap) {
  if ((ULONG_PTR)s <= 0xFFFF) {
    wsprintfA(out, "#%u", (unsigned int)(ULONG_PTR)s);
    return;
  }
  int n = WideCharToMultiByte(CP_UTF8, 0, s, -1, out, (int)out_cap, NULL, NULL);
  if (n <= 0)
    out[0] = '\0';
}

BOOL is_cef_subprocess(void) {
  const char *cmdline = GetCommandLineA();
  return cmdline && ansi_contains_ci(cmdline, "--type=");
}

BOOL running_as_wallpaper64(void) {
  WCHAR path[MAX_PATH];
  DWORD len = GetModuleFileNameW(NULL, path, MAX_PATH);
  if (len == 0 || len >= MAX_PATH)
    return FALSE;
  return wide_contains_ci(path, L"wallpaper64.exe");
}

BOOL names_equal(const char *a, const char *b) {
  while (*a && *a == *b) {
    a++;
    b++;
  }
  return *a == *b;
}

void spawn_dump_log(const char *msg) {
  HANDLE h = CreateFileA("C:\\wallpiper_spawn_dump.log", FILE_APPEND_DATA,
                         FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_ALWAYS,
                         FILE_ATTRIBUTE_NORMAL, NULL);
  if (h == INVALID_HANDLE_VALUE)
    return;
  DWORD written;
  WriteFile(h, msg, (DWORD)lstrlenA(msg), &written, NULL);
  WriteFile(h, "\r\n", 2, &written, NULL);
  CloseHandle(h);
}
