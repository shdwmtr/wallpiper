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

#ifndef WP_UTIL_H
#define WP_UTIL_H

#include "windows.h"

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
BOOL running_as_wallpaper_engine(void);
BOOL names_equal(const char *a, const char *b);
void spawn_dump_log(const char *msg);

#endif
