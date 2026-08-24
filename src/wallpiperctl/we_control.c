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

#include "we_control.h"

#include <ctype.h>
#include <dirent.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

#include "cJSON.h"
#include "wallpiper/steam_paths.h"

#define WE_MAX_CONTROL_ARGS 10

bool wp_we_send_control(const char *const *control_args,
                        size_t control_arg_count, char *err_out,
                        size_t err_out_len) {
  if (control_arg_count > WE_MAX_CONTROL_ARGS) {
    snprintf(err_out, err_out_len, "too many control arguments");
    return false;
  }

  char wine_bin[1024];
  char we_exe[1024];
  char compatdata[768];
  if (!wp_wine_bin(wine_bin, sizeof(wine_bin), err_out, err_out_len)) {
    return false;
  }
  if (!wp_we_exe(we_exe, sizeof(we_exe), err_out, err_out_len)) {
    return false;
  }
  if (!wp_compatdata_dir(compatdata, sizeof(compatdata), err_out,
                         err_out_len)) {
    return false;
  }

  char pfx[800];
  int n = snprintf(pfx, sizeof(pfx), "%s/pfx", compatdata);
  if (n <= 0 || (size_t)n >= sizeof(pfx)) {
    snprintf(err_out, err_out_len, "compatdata path too long");
    return false;
  }

  pid_t pid = fork();
  if (pid < 0) {
    snprintf(err_out, err_out_len, "fork failed");
    return false;
  }

  if (pid == 0) {
    setenv("WINEPREFIX", pfx, 1);

    int devnull = open("/dev/null", O_WRONLY);
    if (devnull >= 0) {
      dup2(devnull, STDOUT_FILENO);
      dup2(devnull, STDERR_FILENO);
      close(devnull);
    }

    const char *argv[3 + WE_MAX_CONTROL_ARGS + 1];
    size_t i = 0;
    argv[i++] = wine_bin;
    argv[i++] = we_exe;
    argv[i++] = "-control";
    for (size_t j = 0; j < control_arg_count; j++) {
      argv[i++] = control_args[j];
    }
    argv[i] = NULL;

    execv(wine_bin, (char *const *)argv);
    _exit(127);
  }

  int status = 0;
  if (waitpid(pid, &status, 0) < 0) {
    snprintf(err_out, err_out_len, "failed to wait for wine process");
    return false;
  }

  if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
    snprintf(err_out, err_out_len,
             "wallpaper engine did not respond to the control command (is "
             "it running?)");
    return false;
  }
  return true;
}

static bool resolve_project_json(const char *target, char *out,
                                 size_t out_len, char *err_out,
                                 size_t err_out_len) {
  bool is_workshop_id = target[0] != '\0';
  for (const char *p = target; *p; p++) {
    if (!isdigit((unsigned char)*p)) {
      is_workshop_id = false;
      break;
    }
  }

  char resolved[1024];
  if (is_workshop_id) {
    char workshop_dir[900];
    if (!wp_workshop_content_dir(workshop_dir, sizeof(workshop_dir), err_out,
                                 err_out_len)) {
      return false;
    }
    int n = snprintf(resolved, sizeof(resolved), "%s/%s", workshop_dir, target);
    if (n <= 0 || (size_t)n >= sizeof(resolved)) {
      snprintf(err_out, err_out_len, "path too long");
      return false;
    }
  } else {
    char *real = realpath(target, NULL);
    if (!real) {
      snprintf(err_out, err_out_len, "no such file or directory: %s", target);
      return false;
    }
    int n = snprintf(resolved, sizeof(resolved), "%s", real);
    free(real);
    if (n <= 0 || (size_t)n >= sizeof(resolved)) {
      snprintf(err_out, err_out_len, "path too long");
      return false;
    }
  }

  struct stat st;
  if (stat(resolved, &st) != 0) {
    snprintf(err_out, err_out_len, "wallpaper not found: %s", resolved);
    return false;
  }

  if (S_ISDIR(st.st_mode)) {
    size_t base_len = strlen(resolved);
    int n = snprintf(resolved + base_len, sizeof(resolved) - base_len,
                     "/project.json");
    if (n <= 0 || (size_t)n >= sizeof(resolved) - base_len) {
      snprintf(err_out, err_out_len, "path too long");
      return false;
    }
    if (stat(resolved, &st) != 0) {
      snprintf(err_out, err_out_len, "no project.json found in %s", resolved);
      return false;
    }
  }

  if (strlen(resolved) >= out_len) {
    snprintf(err_out, err_out_len, "path too long");
    return false;
  }
  strcpy(out, resolved);
  return true;
}

bool wp_we_set_wallpaper(const char *target, const char *monitor,
                         char *err_out, size_t err_out_len) {
  char resolved[1024];
  if (!resolve_project_json(target, resolved, sizeof(resolved), err_out,
                            err_out_len)) {
    return false;
  }

  char win_path[1030];
  if (!wp_to_windows_path(resolved, win_path, sizeof(win_path))) {
    snprintf(err_out, err_out_len, "path too long to convert");
    return false;
  }

  const char *slot = (monitor && monitor[0] != '\0') ? monitor : "0";

  const char *control_args[] = {"openWallpaper", "-file", win_path,
                                "-monitor",      slot};
  return wp_we_send_control(control_args, 5, err_out, err_out_len);
}

bool wp_we_set_property(const char *target, const char *name,
                        const char *value, const char *monitor,
                        char *err_out, size_t err_out_len) {
  char resolved[1024];
  if (!resolve_project_json(target, resolved, sizeof(resolved), err_out,
                            err_out_len)) {
    return false;
  }

  char win_path[1030];
  if (!wp_to_windows_path(resolved, win_path, sizeof(win_path))) {
    snprintf(err_out, err_out_len, "path too long to convert");
    return false;
  }

  const char *slot = (monitor && monitor[0] != '\0') ? monitor : "0";

  const char *control_args[] = {"setProperty", "-file", win_path, "-name",
                                name,          "-value", value,   "-monitor",
                                slot};
  return wp_we_send_control(control_args, 9, err_out, err_out_len);
}

bool wp_we_list_wallpapers(char *err_out, size_t err_out_len) {
  char workshop_dir[900];
  if (!wp_workshop_content_dir(workshop_dir, sizeof(workshop_dir), err_out,
                               err_out_len)) {
    return false;
  }

  DIR *d = opendir(workshop_dir);
  if (!d) {
    snprintf(err_out, err_out_len, "could not open workshop content dir %s",
             workshop_dir);
    return false;
  }

  struct dirent *entry;
  while ((entry = readdir(d)) != NULL) {
    if (entry->d_name[0] == '.') {
      continue;
    }

    char project_path[1024];
    int n = snprintf(project_path, sizeof(project_path), "%s/%s/project.json",
                     workshop_dir, entry->d_name);
    if (n <= 0 || (size_t)n >= sizeof(project_path)) {
      continue;
    }

    FILE *f = fopen(project_path, "rb");
    if (!f) {
      continue;
    }

    char buf[8192];
    size_t len = fread(buf, 1, sizeof(buf) - 1, f);
    fclose(f);
    buf[len] = '\0';

    cJSON *root = cJSON_ParseWithLength(buf, len);
    const char *title = entry->d_name;
    if (root) {
      cJSON *title_node = cJSON_GetObjectItemCaseSensitive(root, "title");
      if (cJSON_IsString(title_node) && title_node->valuestring) {
        title = title_node->valuestring;
      }
    }
    printf("%-12s %s\n", entry->d_name, title);
    if (root) {
      cJSON_Delete(root);
    }
  }
  closedir(d);
  return true;
}
