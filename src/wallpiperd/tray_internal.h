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

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define WP_TRAY_MAX_ENTRIES 512
#define WP_TRAY_MFT_SEPARATOR 0x800u
#define WP_TRAY_MFS_DISABLED 0x3u
#define WP_TRAY_MFS_CHECKED 0x8u

typedef struct {
  int32_t depth;
  uint32_t id;
  uint32_t state;
  uint32_t type_flags;
  char text[256];
} wp_tray_entry_t;

typedef struct {
  wp_tray_entry_t entries[WP_TRAY_MAX_ENTRIES];
  size_t count;
} wp_tray_entries_t;

void wp_tray_parse_menu_dump(const char *text, wp_tray_entries_t *out);

typedef struct {
  int32_t width;
  int32_t height;
  char tooltip[256];
  uint8_t *pixels_argb;
} wp_tray_icon_t;

bool wp_tray_parse_icon_file(const uint8_t *buf, size_t len,
                             wp_tray_icon_t *out);
void wp_tray_icon_release(wp_tray_icon_t *icon);

void wp_tray_write_click(uint32_t event_code);
void wp_tray_send_menu_command(uint32_t id);

void wp_tray_debug_log(const char *fmt, ...);

void wp_tray_files_spawn_watchers(void);

void wp_tray_state_on_menu_dump_changed(const wp_tray_entries_t *entries);
void wp_tray_state_on_icon_changed(wp_tray_icon_t *icon);

void wp_tray_dbus_start(void);
