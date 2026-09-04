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

#include "vk_layer.h"

#include <stdio.h>
#include <sys/stat.h>

#include "wallpiper/fsutil.h"
#include "wallpiper/protocol.h"
#include "wallpiper/steam_paths.h"

static bool path_exists(const char *path) {
  struct stat st;
  return stat(path, &st) == 0 && S_ISREG(st.st_mode);
}

bool wp_vk_capture_layer32_available(void) {
  char install_dir[1024];
  if (!wp_install_dir(install_dir, sizeof(install_dir))) {
    return false;
  }
  char path[1100];
  int n = snprintf(path, sizeof(path), "%s/libVkLayer_wallpiper_capture32.so",
                   install_dir);
  if (n <= 0 || (size_t)n >= sizeof(path)) {
    return false;
  }
  return path_exists(path);
}

static void write_manifest(const char *temp_dir, const char *install_dir,
                           const char *manifest_name, const char *layer_name,
                           const char *so_filename) {
  char path[600];
  int n = snprintf(path, sizeof(path), "%s/%s.json", temp_dir, manifest_name);
  if (n <= 0 || (size_t)n >= sizeof(path)) {
    printf("vk layer manifest path too long\n");
    return;
  }

  FILE *f = fopen(path, "w");
  if (!f) {
    printf("failed to write vk layer manifest at %s\n", path);
    return;
  }

  fprintf(f,
          "{\n"
          "    \"file_format_version\" : \"1.0.0\",\n"
          "    \"layer\" : {\n"
          "        \"name\": \"%s\",\n"
          "        \"type\": \"GLOBAL\",\n"
          "        \"library_path\": \"%s/%s\",\n"
          "        \"api_version\": \"1.1.0\",\n"
          "        \"implementation_version\": \"1\",\n"
          "        \"description\": \"Wallpiper frame capture layer\"\n"
          "    }\n"
          "}\n",
          layer_name, install_dir, so_filename);
  fclose(f);
}

void wp_write_vk_layer_manifest(void) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    printf("failed to resolve temp dir for vk layer manifest\n");
    return;
  }
  wp_mkdir_p(temp_dir);

  char install_dir[1024];
  if (!wp_install_dir(install_dir, sizeof(install_dir))) {
    printf("failed to resolve install dir for vk layer manifest\n");
    return;
  }

  write_manifest(temp_dir, install_dir, WP_VK_CAPTURE_LAYER_NAME,
                WP_VK_CAPTURE_LAYER_NAME, "libVkLayer_wallpiper_capture.so");

  if (wp_vk_capture_layer32_available()) {
    write_manifest(temp_dir, install_dir, WP_VK_CAPTURE_LAYER_NAME_32,
                  WP_VK_CAPTURE_LAYER_NAME_32,
                  "libVkLayer_wallpiper_capture32.so");
  }
}
