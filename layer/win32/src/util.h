#ifndef WP_UTIL_H
#define WP_UTIL_H

#include "win32/windows.h"

void debug_log(const char *msg);
void ipc_dump_log(const char *msg);
void window_dump_log(const char *msg);
void menu_build_log(const char *msg);
UINT64 unix_millis(void);

BOOL wide_contains_ci_len(const WCHAR *haystack, size_t hlen,
                          const WCHAR *needle);
BOOL wide_contains_ci(const WCHAR *haystack, const WCHAR *needle);
BOOL ansi_contains_ci(const char *haystack, const char *needle);
BOOL ansi_starts_with_ci(const char *s, const char *prefix);
BOOL wide_starts_with_ci(const WCHAR *s, const WCHAR *prefix);
void basename_w(const WCHAR *path, WCHAR *out);
BOOL get_env_path(const WCHAR *name, WCHAR *out, DWORD out_len);
void narrow_maybe_atom(LPCWSTR s, char *out, size_t out_cap);
BOOL is_cef_subprocess(void);
BOOL running_as_wallpaper64(void);
BOOL names_equal(const char *a, const char *b);
void spawn_dump_log(const char *msg);

#endif
