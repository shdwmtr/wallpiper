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

#include <dlfcn.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#include "log.h"

#if defined(__GNUC__) || defined(__clang__)
#define WP_EXPORT __attribute__((visibility("default")))
#else
#define WP_EXPORT
#endif

typedef int (*pfn_symlink)(const char *, const char *);

static void wp_normalize_path(const char *in, char *out, size_t out_size) {
  size_t o = 0;
  bool prev_slash = false;
  for (size_t i = 0; in[i] != '\0' && o + 1 < out_size; i++) {
    char c = in[i];
    if (c == '/') {
      if (prev_slash) {
        continue;
      }
      prev_slash = true;
    } else {
      prev_slash = false;
    }
    out[o++] = c;
  }
  while (o > 1 && out[o - 1] == '/') {
    o--;
  }
  out[o] = '\0';
}

static const char *wp_font_redirect_target(const char *linkpath) {
  const char *redirects = getenv("WALLPIPER_FONT_REDIRECTS");
  if (!redirects || !*redirects) {
    return NULL;
  }

  char normalized_linkpath[4096];
  wp_normalize_path(linkpath, normalized_linkpath, sizeof(normalized_linkpath));

  char *copy = strdup(redirects);
  if (!copy) {
    return NULL;
  }

  static char target[4096];
  const char *result = NULL;

  char *saveptr = NULL;
  for (char *entry = strtok_r(copy, ";", &saveptr); entry;
       entry = strtok_r(NULL, ";", &saveptr)) {
    char *eq = strchr(entry, '=');
    if (!eq) {
      continue;
    }
    *eq = '\0';
    const char *entry_target = eq + 1;

    char normalized_entry[4096];
    wp_normalize_path(entry, normalized_entry, sizeof(normalized_entry));

    if (strcmp(normalized_entry, normalized_linkpath) == 0) {
      strncpy(target, entry_target, sizeof(target) - 1);
      target[sizeof(target) - 1] = '\0';
      result = target;
      break;
    }
  }

  free(copy);
  return result;
}

WP_EXPORT int symlink(const char *target, const char *linkpath) {
  pfn_symlink real = (pfn_symlink)dlsym(RTLD_NEXT, "symlink");
  if (!real) {
    return -1;
  }

  const char *redirect = wp_font_redirect_target(linkpath);
  if (redirect) {
    wp_log("font redirect: %s -> %s (was %s)", linkpath, redirect, target);
    return real(redirect, linkpath);
  }

  return real(target, linkpath);
}
