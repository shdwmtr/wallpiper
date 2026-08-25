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

#define WALLPAPER_ENGINE_APP_ID "431960"

bool wp_home_dir(char *out, size_t out_len);
bool wp_install_dir(char *out, size_t out_len);

bool wp_steam_root(char *out, size_t out_len, char *err_out,
                   size_t err_out_len);
bool wp_compatdata_dir(char *out, size_t out_len, char *err_out,
                       size_t err_out_len);
bool wp_workshop_content_dir(char *out, size_t out_len, char *err_out,
                             size_t err_out_len);
bool wp_we_config_path(char *out, size_t out_len, char *err_out,
                       size_t err_out_len);
bool wp_we_ensure_default_config(char *err_out, size_t err_out_len);
bool wp_we_exe(char *out, size_t out_len, char *err_out, size_t err_out_len);
bool wp_we_sync_distribution(char *err_out, size_t err_out_len);
bool wp_proton_bin(char *out, size_t out_len, char *err_out,
                   size_t err_out_len);
bool wp_wine_bin(char *out, size_t out_len, char *err_out, size_t err_out_len);
bool wp_portal_name(char *out, size_t out_len, char *err_out,
                    size_t err_out_len);

bool wp_to_windows_path(const char *unix_path, char *out, size_t out_len);
bool wp_from_windows_path(const char *windows_path, char *out, size_t out_len);

void wp_describe(void);
