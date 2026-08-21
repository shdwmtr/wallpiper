#include "renderer.h"

#include "config.h"
#include "dwmapi_shim.h"
#include "fonts.h"
#include "process.h"
#include "vk_layer.h"

#include <fcntl.h>
#include <math.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#include "wallpiper/fsutil.h"
#include "wallpiper/protocol.h"
#include "wallpiper/steam_paths.h"

static bool ensure_compatdata_dir(const char *location, char *out,
                                  size_t out_len) {
  char err[256];
  if (!wp_compatdata_for(location, out, out_len, err, sizeof(err))) {
    return false;
  }
  if (!wp_mkdir_p(out)) {
    printf("failed to create compatdata dir %s\n", out);
  }
  return true;
}

static void set_proton_env(const char *compatdata) {
  char steam_root[768];
  char err[256];
  if (wp_steam_root(steam_root, sizeof(steam_root), err, sizeof(err))) {
    setenv("STEAM_COMPAT_CLIENT_INSTALL_PATH", steam_root, 1);
  }
  setenv("STEAM_COMPAT_DATA_PATH", compatdata, 1);
  setenv("SteamAppId", WALLPAPER_ENGINE_APP_ID, 1);
  setenv("SteamGameId", WALLPAPER_ENGINE_APP_ID, 1);
}

static bool wineserver_bin(char *out, size_t out_len) {
  char proton[1024];
  char err[256];
  if (!wp_proton_bin(proton, sizeof(proton), err, sizeof(err))) {
    return false;
  }
  char *slash = strrchr(proton, '/');
  if (!slash) {
    return false;
  }
  *slash = '\0';

  char candidate[1200];
  snprintf(candidate, sizeof(candidate), "%s/files/bin/wineserver", proton);

  struct stat st;
  if (stat(candidate, &st) != 0 || !S_ISREG(st.st_mode)) {
    return false;
  }
  return snprintf(out, out_len, "%s", candidate) > 0;
}

static uint32_t target_log_pixels(double scale) {
  if (!(scale > 0.0)) {
    scale = 1.0;
  }
  long value = lround(96.0 * scale);
  if (value < 96) {
    value = 96;
  }
  if (value > 480) {
    value = 480;
  }
  return (uint32_t)value;
}

static bool read_applied_log_pixels(const char *location, uint32_t *out) {
  char path[1024];
  if (!wp_dpi_marker_path(location, path, sizeof(path))) {
    return false;
  }
  FILE *f = fopen(path, "r");
  if (!f) {
    return false;
  }
  char buf[64];
  size_t n = fread(buf, 1, sizeof(buf) - 1, f);
  fclose(f);
  buf[n] = '\0';

  char *end = NULL;
  unsigned long v = strtoul(buf, &end, 10);
  if (end == buf) {
    return false;
  }
  *out = (uint32_t)v;
  return true;
}

static void write_applied_log_pixels(const char *location, uint32_t value) {
  char path[1024];
  if (!wp_dpi_marker_path(location, path, sizeof(path))) {
    return;
  }
  FILE *f = fopen(path, "w");
  if (!f) {
    return;
  }
  fprintf(f, "%u", value);
  fclose(f);
}

static bool dpi_change_needed(const char *location, double scale,
                              uint32_t *out_target) {
  uint32_t target = target_log_pixels(scale);
  uint32_t applied;
  bool has_applied = read_applied_log_pixels(location, &applied);
  if (has_applied && applied == target) {
    return false;
  }
  *out_target = target;
  return true;
}

static void apply_dpi(const char *location, uint32_t target) {
  char proton[1024];
  char err[256];
  if (!wp_proton_bin(proton, sizeof(proton), err, sizeof(err))) {
    printf("failed to run wine reg add: %s, leaving DPI as-is for %s\n", err,
           location);
    return;
  }

  char compatdata[768];
  if (!ensure_compatdata_dir(location, compatdata, sizeof(compatdata))) {
    return;
  }

  char target_str[16];
  snprintf(target_str, sizeof(target_str), "%u", target);

  pid_t pid = fork();
  if (pid < 0) {
    printf("failed to run wine reg add, leaving DPI as-is for %s\n", location);
    return;
  }
  if (pid == 0) {
    set_proton_env(compatdata);
    execl(proton, proton, "runinprefix", "reg", "add",
          "HKCU\\Control Panel\\Desktop", "/v", "LogPixels", "/t", "REG_DWORD",
          "/d", target_str, "/f", (char *)NULL);
    _exit(127);
  }

  int status = 0;
  waitpid(pid, &status, 0);
  if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
    write_applied_log_pixels(location, target);
    printf("applied LogPixels=%u for %s\n", target, location);
  } else {
    printf("wine reg add exited abnormally, leaving DPI as-is for %s\n",
           location);
  }
}

static void kill_prefix_session(const char *location) {
  wp_pid_list_t existing;
  wp_find_renderer_pids_for_location(location, &existing);
  if (existing.count > 0) {
    wp_kill_pids_gracefully(existing.pids, existing.count);
  }

  char compatdata[768];
  if (!ensure_compatdata_dir(location, compatdata, sizeof(compatdata))) {
    return;
  }

  char wineserver[1200];
  if (!wineserver_bin(wineserver, sizeof(wineserver))) {
    printf("could not locate wineserver next to proton binary, skipping "
           "explicit kill for %s\n",
           location);
    return;
  }

  char wineprefix[900];
  snprintf(wineprefix, sizeof(wineprefix), "%s/pfx", compatdata);

  pid_t pid = fork();
  if (pid < 0) {
    printf("failed to stop wineserver for %s, continuing anyway\n", location);
    return;
  }
  if (pid == 0) {
    setenv("WINEPREFIX", wineprefix, 1);
    execl(wineserver, wineserver, "-k", "-w", (char *)NULL);
    _exit(127);
  }
  int status = 0;
  waitpid(pid, &status, 0);
}

static void ensure_prefix_configured(const char *location, double scale) {
  uint32_t target;
  if (!dpi_change_needed(location, scale, &target)) {
    return;
  }
  printf("prefix config changed for %s, restarting renderer\n", location);
  kill_prefix_session(location);
  apply_dpi(location, target);
}

void wp_renderer_set_paused(bool paused) {
  int pid = wp_find_renderer_pid();
  if (pid < 0) {
    printf("%s: no active renderer found\n", paused ? "pause" : "resume");
    return;
  }
  int sig = paused ? SIGSTOP : SIGCONT;
  int res = kill(pid, sig);
  printf("%s renderer pid=%d -> %s\n", paused ? "paused" : "resumed", pid,
         res == 0 ? "ok" : "failed");
}

static bool renderer_pid_still_valid(int pid) {
  char path[64];
  snprintf(path, sizeof(path), "/proc/%d/comm", pid);
  FILE *f = fopen(path, "r");
  if (!f) {
    return false;
  }
  char comm[256];
  size_t n = fread(comm, 1, sizeof(comm) - 1, f);
  fclose(f);
  while (n > 0 && (comm[n - 1] == '\n' || comm[n - 1] == '\r')) {
    n--;
  }
  comm[n] = '\0';
  return strcmp(comm, "wallpaper64.exe") == 0;
}

static void read_tracked_renderer_pids(const char *location,
                                       wp_pid_list_t *out) {
  out->count = 0;

  char path[1024];
  if (!wp_renderer_pid_path(location, path, sizeof(path))) {
    return;
  }
  FILE *f = fopen(path, "r");
  if (!f) {
    return;
  }

  char line[64];
  while (fgets(line, sizeof(line), f) &&
         out->count < sizeof(out->pids) / sizeof(out->pids[0])) {
    char *end = NULL;
    long pid = strtol(line, &end, 10);
    if (end != line && pid > 0) {
      out->pids[out->count++] = (int)pid;
    }
  }
  fclose(f);
}

static void write_tracked_renderer_pids(const char *location,
                                        const wp_pid_list_t *pids) {
  char path[1024];
  if (!wp_renderer_pid_path(location, path, sizeof(path))) {
    return;
  }
  FILE *f = fopen(path, "w");
  if (!f) {
    return;
  }
  for (size_t i = 0; i < pids->count; i++) {
    fprintf(f, "%s%d", i == 0 ? "" : "\n", pids->pids[i]);
  }
  fclose(f);
}

static bool pid_list_contains(const wp_pid_list_t *list, int pid) {
  for (size_t i = 0; i < list->count; i++) {
    if (list->pids[i] == pid) {
      return true;
    }
  }
  return false;
}

static bool discover_new_renderer_pid(const char *location,
                                      const wp_pid_list_t *pre_spawn,
                                      int *out_pid) {
  for (int attempt = 0; attempt < 20; attempt++) {
    wp_pid_list_t current;
    wp_find_renderer_pids_for_location(location, &current);
    for (size_t i = 0; i < current.count; i++) {
      if (!pid_list_contains(pre_spawn, current.pids[i])) {
        printf("tracking new renderer pid=%d for %s\n", current.pids[i],
               location);
        *out_pid = current.pids[i];
        return true;
      }
    }
    struct timespec ts = {.tv_sec = 0, .tv_nsec = 250000000L};
    nanosleep(&ts, NULL);
  }
  printf("timed out waiting for new renderer process to appear for %s\n",
         location);
  return false;
}

void wp_renderer_spawn(const char *location, wp_monitor_geometry_t monitor) {
  printf("spawning renderer: location=%s (wallpaper selection is Wallpaper "
         "Engine's own) %ux%u scale=%g\n",
         location, monitor.width, monitor.height, monitor.scale);

  ensure_prefix_configured(location, monitor.scale);

  wp_pid_list_t old_wrappers;
  wp_find_proton_wrapper_pids_for_location(location, &old_wrappers);

  wp_pid_list_t pending_renderers;
  read_tracked_renderer_pids(location, &pending_renderers);

  wp_pid_list_t pre_spawn_renderers;
  wp_find_renderer_pids_for_location(location, &pre_spawn_renderers);

  char logpath[512];
  snprintf(logpath, sizeof(logpath), "/tmp/wallpiperd-%s.log", location);
  int logfd = open(logpath, O_WRONLY | O_CREAT | O_TRUNC, 0644);
  if (logfd < 0) {
    printf("failed to create renderer logfile for %s\n", location);
    return;
  }

  char proton[1024];
  char err[256];
  if (!wp_proton_bin(proton, sizeof(proton), err, sizeof(err))) {
    printf("failed to spawn: %s\n", err);
    close(logfd);
    return;
  }
  char we_exe[1024];
  if (!wp_we_exe(we_exe, sizeof(we_exe), err, sizeof(err))) {
    printf("failed to spawn: %s\n", err);
    close(logfd);
    return;
  }
  char compatdata[768];
  if (!ensure_compatdata_dir(location, compatdata, sizeof(compatdata))) {
    close(logfd);
    return;
  }

  pid_t pid = fork();
  if (pid < 0) {
    printf("failed to spawn: fork failed\n");
    close(logfd);
    return;
  }

  if (pid == 0) {
    dup2(logfd, STDOUT_FILENO);
    dup2(logfd, STDERR_FILENO);
    close(logfd);
    setpgid(0, 0);

    set_proton_env(compatdata);
    wp_dwmapi_shim_wire_up(location);

    char preload[1024];
    if (wp_preload_path(preload, sizeof(preload))) {
      setenv("LD_PRELOAD", preload, 1);
    }

    char font_redirects[16384];
    if (wp_fonts_env_value(location, font_redirects, sizeof(font_redirects))) {
      setenv("WALLPIPER_FONT_REDIRECTS", font_redirects, 1);
    }

    char guardian_pid[16];
    snprintf(guardian_pid, sizeof(guardian_pid), "%d", (int)getppid());
    setenv("WALLPIPER_GUARDIAN_PID", guardian_pid, 1);

    char vk_layer_path[512];
    if (wp_vk_layer_path(vk_layer_path, sizeof(vk_layer_path))) {
      setenv("VK_ADD_LAYER_PATH", vk_layer_path, 1);
    }
    setenv("VK_INSTANCE_LAYERS", WP_VK_CAPTURE_LAYER_NAME, 1);

    char num[32];
    snprintf(num, sizeof(num), "%d", monitor.x);
    setenv("WALLPIPER_MONITOR_X", num, 1);
    snprintf(num, sizeof(num), "%d", monitor.y);
    setenv("WALLPIPER_MONITOR_Y", num, 1);
    snprintf(num, sizeof(num), "%u", monitor.width);
    setenv("WALLPIPER_MONITOR_WIDTH", num, 1);
    snprintf(num, sizeof(num), "%u", monitor.height);
    setenv("WALLPIPER_MONITOR_HEIGHT", num, 1);

    char portal_name[64];
    char perr[256];
    if (wp_portal_name(portal_name, sizeof(portal_name), perr, sizeof(perr))) {
      char ctl_socket[256];
      if (wp_ctl_socket_path(portal_name, ctl_socket, sizeof(ctl_socket))) {
        setenv("WALLPIPER_PORTAL_CTL_SOCKET", ctl_socket, 1);
      }
    }

    char cursor_path[1024];
    if (wp_cursor_pos_path(location, cursor_path, sizeof(cursor_path))) {
      setenv("WALLPIPER_CURSOR_POS_FILE", cursor_path, 1);
    }

    char windowbrowser_socket[512];
    if (wp_windowbrowser_socket_path(windowbrowser_socket,
                                     sizeof(windowbrowser_socket))) {
      setenv("WALLPIPER_WINDOWBROWSER_SOCKET", windowbrowser_socket, 1);
    }

    char win_path[1024];
    char win_out[1030];
    if (wp_tray_icon_path(win_path, sizeof(win_path)) &&
        wp_to_windows_path(win_path, win_out, sizeof(win_out))) {
      setenv("WALLPIPER_TRAY_ICON_FILE", win_out, 1);
    }
    if (wp_tray_click_path(win_path, sizeof(win_path)) &&
        wp_to_windows_path(win_path, win_out, sizeof(win_out))) {
      setenv("WALLPIPER_TRAY_CLICK_FILE", win_out, 1);
    }
    if (wp_menu_file_path(win_path, sizeof(win_path)) &&
        wp_to_windows_path(win_path, win_out, sizeof(win_out))) {
      setenv("WALLPIPER_MENU_FILE", win_out, 1);
    }
    if (wp_menu_command_path(win_path, sizeof(win_path)) &&
        wp_to_windows_path(win_path, win_out, sizeof(win_out))) {
      setenv("WALLPIPER_MENU_COMMAND_FILE", win_out, 1);
    }

    execl(proton, proton, "run", we_exe, (char *)NULL);
    _exit(127);
  }

  close(logfd);
  printf("spawned proton wrapper pid=%d\n", (int)pid);

  if (old_wrappers.count > 0) {
    printf("cleaning up old proton wrapper pid(s) for %s\n", location);
    wp_kill_pids_gracefully(old_wrappers.pids, old_wrappers.count);
  }

  wp_pid_list_t still_valid_pending;
  still_valid_pending.count = 0;
  for (size_t i = 0; i < pending_renderers.count; i++) {
    if (renderer_pid_still_valid(pending_renderers.pids[i])) {
      still_valid_pending.pids[still_valid_pending.count++] =
          pending_renderers.pids[i];
    }
  }
  pending_renderers = still_valid_pending;

  if (pending_renderers.count > 0) {
    wp_pid_list_t pickers;
    wp_find_picker_pids(&pickers);
    if (pickers.count == 0) {
      printf("no picker session open, cleaning up deferred renderer pid(s) for "
             "%s\n",
             location);
      wp_kill_pids_gracefully(pending_renderers.pids, pending_renderers.count);
      pending_renderers.count = 0;
    } else {
      printf(
          "picker session open, deferring cleanup of renderer pid(s) for %s\n",
          location);
    }
  }

  int new_pid;
  if (discover_new_renderer_pid(location, &pre_spawn_renderers, &new_pid)) {
    if (pending_renderers.count <
        sizeof(pending_renderers.pids) / sizeof(pending_renderers.pids[0])) {
      pending_renderers.pids[pending_renderers.count++] = new_pid;
    }
  }
  write_tracked_renderer_pids(location, &pending_renderers);
}

void wp_renderer_swap(const char *location, wp_monitor_geometry_t monitor) {
  wp_pid_list_t existing;
  wp_find_renderer_pids_for_location(location, &existing);
  if (existing.count == 0) {
    wp_renderer_spawn(location, monitor);
  }
}
