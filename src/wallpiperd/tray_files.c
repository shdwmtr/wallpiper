#include "tray_internal.h"

#include "config.h"

#include <fcntl.h>
#include <pthread.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/inotify.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

static bool parse_hex_field(const char *s, size_t len, uint32_t *out) {
  char buf[32];
  if (len >= sizeof(buf)) {
    return false;
  }
  memcpy(buf, s, len);
  buf[len] = '\0';

  const char *p = buf;
  if (p[0] == '0' && (p[1] == 'x' || p[1] == 'X')) {
    p += 2;
  }

  char *end = NULL;
  unsigned long v = strtoul(p, &end, 16);
  if (end == p) {
    return false;
  }
  *out = (uint32_t)v;
  return true;
}

void wp_tray_parse_menu_dump(const char *text, wp_tray_entries_t *out) {
  out->count = 0;

  const char *line_start = text;
  while (*line_start) {
    const char *line_end = strchr(line_start, '\n');
    size_t line_len =
        line_end ? (size_t)(line_end - line_start) : strlen(line_start);
    size_t eff_len = line_len;
    if (eff_len > 0 && line_start[eff_len - 1] == '\r') {
      eff_len--;
    }

    if (eff_len > 0 && out->count < WP_TRAY_MAX_ENTRIES) {
      const char *p = line_start;
      const char *end = line_start + eff_len;

      const char *t1 = memchr(p, '\t', (size_t)(end - p));
      const char *t2 =
          t1 ? memchr(t1 + 1, '\t', (size_t)(end - (t1 + 1))) : NULL;
      const char *t3 =
          t2 ? memchr(t2 + 1, '\t', (size_t)(end - (t2 + 1))) : NULL;
      const char *t4 =
          t3 ? memchr(t3 + 1, '\t', (size_t)(end - (t3 + 1))) : NULL;

      if (t4) {
        char depth_buf[32];
        char id_buf[32];
        size_t depth_len = (size_t)(t1 - p);
        size_t id_len = (size_t)(t2 - (t1 + 1));
        uint32_t state_v, type_v;

        if (depth_len < sizeof(depth_buf) && id_len < sizeof(id_buf) &&
            parse_hex_field(t2 + 1, (size_t)(t3 - (t2 + 1)), &state_v) &&
            parse_hex_field(t3 + 1, (size_t)(t4 - (t3 + 1)), &type_v)) {
          memcpy(depth_buf, p, depth_len);
          depth_buf[depth_len] = '\0';
          memcpy(id_buf, t1 + 1, id_len);
          id_buf[id_len] = '\0';

          char *dend = NULL;
          long depth_v = strtol(depth_buf, &dend, 10);
          char *iend = NULL;
          unsigned long id_v = strtoul(id_buf, &iend, 10);

          if (dend != depth_buf && iend != id_buf) {
            wp_tray_entry_t *e = &out->entries[out->count];
            e->depth = (int32_t)depth_v;
            e->id = (uint32_t)id_v;
            e->state = state_v;
            e->type_flags = type_v;

            size_t text_len = (size_t)(end - (t4 + 1));
            if (text_len >= sizeof(e->text)) {
              text_len = sizeof(e->text) - 1;
            }
            memcpy(e->text, t4 + 1, text_len);
            e->text[text_len] = '\0';
            out->count++;
          }
        }
      }
    }

    if (!line_end) {
      break;
    }
    line_start = line_end + 1;
  }
}

static uint32_t read_u32le(const uint8_t *buf, size_t off) {
  return (uint32_t)buf[off] | ((uint32_t)buf[off + 1] << 8) |
         ((uint32_t)buf[off + 2] << 16) | ((uint32_t)buf[off + 3] << 24);
}

bool wp_tray_parse_icon_file(const uint8_t *buf, size_t len,
                             wp_tray_icon_t *out) {
  if (len < 36) {
    return false;
  }
  if (read_u32le(buf, 0) != 0x59415254u) {
    return false;
  }

  int32_t width = (int32_t)read_u32le(buf, 24);
  int32_t height = (int32_t)read_u32le(buf, 28);
  uint32_t tooltip_len = read_u32le(buf, 32);

  size_t tooltip_start = 36;
  if (tooltip_start + tooltip_len > len) {
    return false;
  }

  size_t tt_copy = tooltip_len;
  if (tt_copy >= sizeof(out->tooltip)) {
    tt_copy = sizeof(out->tooltip) - 1;
  }
  memcpy(out->tooltip, buf + tooltip_start, tt_copy);
  out->tooltip[tt_copy] = '\0';

  if (width < 0 || height < 0) {
    return false;
  }

  size_t pixels_start = tooltip_start + tooltip_len;
  size_t pixel_count = (size_t)width * (size_t)height;
  size_t pixel_bytes = pixel_count * 4;
  if (pixels_start + pixel_bytes > len) {
    return false;
  }

  uint8_t *argb = malloc(pixel_bytes > 0 ? pixel_bytes : 1);
  if (!argb) {
    return false;
  }
  const uint8_t *bgra = buf + pixels_start;
  for (size_t i = 0; i < pixel_count; i++) {
    uint8_t b = bgra[i * 4 + 0];
    uint8_t g = bgra[i * 4 + 1];
    uint8_t r = bgra[i * 4 + 2];
    uint8_t a = bgra[i * 4 + 3];
    argb[i * 4 + 0] = a;
    argb[i * 4 + 1] = r;
    argb[i * 4 + 2] = g;
    argb[i * 4 + 3] = b;
  }

  out->width = width;
  out->height = height;
  out->pixels_argb = argb;
  return true;
}

void wp_tray_icon_release(wp_tray_icon_t *icon) {
  free(icon->pixels_argb);
  icon->pixels_argb = NULL;
}

void wp_tray_write_click(uint32_t event_code) {
  char path[1024];
  if (!wp_tray_click_path(path, sizeof(path))) {
    return;
  }
  uint8_t bytes[4] = {
      (uint8_t)event_code,
      (uint8_t)(event_code >> 8),
      (uint8_t)(event_code >> 16),
      (uint8_t)(event_code >> 24),
  };
  FILE *f = fopen(path, "wb");
  if (!f) {
    return;
  }
  fwrite(bytes, 1, sizeof(bytes), f);
  fclose(f);
}

void wp_tray_send_menu_command(uint32_t id) {
  char path[1024];
  if (!wp_menu_command_path(path, sizeof(path))) {
    return;
  }
  uint8_t bytes[4] = {
      (uint8_t)id,
      (uint8_t)(id >> 8),
      (uint8_t)(id >> 16),
      (uint8_t)(id >> 24),
  };
  FILE *f = fopen(path, "wb");
  if (!f) {
    return;
  }
  fwrite(bytes, 1, sizeof(bytes), f);
  fclose(f);
}

void wp_tray_debug_log(const char *fmt, ...) {
  struct timespec ts;
  clock_gettime(CLOCK_REALTIME, &ts);
  long long ms = (long long)ts.tv_sec * 1000 + ts.tv_nsec / 1000000;

  FILE *f = fopen("/tmp/wallpiper-tray-debug.log", "a");
  if (!f) {
    return;
  }
  fprintf(f, "[%lld] ", ms);
  va_list args;
  va_start(args, fmt);
  vfprintf(f, fmt, args);
  va_end(args);
  fprintf(f, "\n");
  fclose(f);
}

static void split_dir_filename(const char *path, char *dir_out,
                               size_t dir_out_len, char *file_out,
                               size_t file_out_len) {
  const char *slash = strrchr(path, '/');
  if (!slash) {
    snprintf(dir_out, dir_out_len, "/tmp");
    snprintf(file_out, file_out_len, "%s", path);
    return;
  }
  size_t dir_len = (size_t)(slash - path);
  if (dir_len == 0) {
    dir_len = 1;
  }
  if (dir_len >= dir_out_len) {
    dir_len = dir_out_len - 1;
  }
  memcpy(dir_out, path, dir_len);
  dir_out[dir_len] = '\0';
  snprintf(file_out, file_out_len, "%s", slash + 1);
}

static char *read_file_to_string(const char *path) {
  FILE *f = fopen(path, "rb");
  if (!f) {
    return NULL;
  }
  if (fseek(f, 0, SEEK_END) != 0) {
    fclose(f);
    return NULL;
  }
  long size = ftell(f);
  if (size < 0) {
    fclose(f);
    return NULL;
  }
  rewind(f);

  char *buf = malloc((size_t)size + 1);
  if (!buf) {
    fclose(f);
    return NULL;
  }
  size_t n = fread(buf, 1, (size_t)size, f);
  fclose(f);
  buf[n] = '\0';
  return buf;
}

static bool inotify_event_matches(const char *buf, ssize_t n,
                                  const char *filename) {
  bool changed = false;
  size_t offset = 0;
  while (offset + sizeof(struct inotify_event) <= (size_t)n) {
    const struct inotify_event *event =
        (const struct inotify_event *)(buf + offset);
    if (event->len > 0 && strcmp(event->name, filename) == 0) {
      changed = true;
    }
    offset += sizeof(struct inotify_event) + event->len;
  }
  return changed;
}

static void *menu_watcher_thread_main(void *arg) {
  (void)arg;

  char path[1024];
  if (!wp_menu_file_path(path, sizeof(path))) {
    return NULL;
  }

  char dir[900];
  char filename[256];
  split_dir_filename(path, dir, sizeof(dir), filename, sizeof(filename));

  int fd = inotify_init1(0);
  if (fd < 0) {
    printf("tray: inotify_init1 failed, native menu mirroring disabled\n");
    return NULL;
  }
  if (inotify_add_watch(fd, dir, IN_CLOSE_WRITE | IN_MOVED_TO) < 0) {
    printf("tray: inotify_add_watch failed for %s, native menu mirroring "
           "disabled\n",
           dir);
    close(fd);
    return NULL;
  }

  wp_tray_debug_log("watcher: started, watching dir=%s for filename=%s", dir,
                    filename);

  char buf[4096];
  for (;;) {
    ssize_t n = read(fd, buf, sizeof(buf));
    if (n <= 0) {
      continue;
    }
    if (!inotify_event_matches(buf, n, filename)) {
      continue;
    }

    char *text = read_file_to_string(path);
    if (!text) {
      wp_tray_debug_log("watcher: matched but read_to_string failed");
      continue;
    }

    wp_tray_entries_t entries;
    wp_tray_parse_menu_dump(text, &entries);
    wp_tray_debug_log("watcher: parsed %zu entries from %zu bytes",
                      entries.count, strlen(text));
    free(text);

    if (entries.count == 0) {
      wp_tray_debug_log("watcher: discarding empty parse (transient rewrite "
                        "race), not publishing");
      continue;
    }

    wp_tray_state_on_menu_dump_changed(&entries);
  }

  return NULL;
}

static bool try_load_icon(const char *path, struct timespec *last_mtime,
                          wp_tray_icon_t *out) {
  struct stat st;
  if (stat(path, &st) != 0) {
    return false;
  }
  if (st.st_mtim.tv_sec == last_mtime->tv_sec &&
      st.st_mtim.tv_nsec == last_mtime->tv_nsec) {
    return false;
  }

  FILE *f = fopen(path, "rb");
  if (!f) {
    return false;
  }
  if (fseek(f, 0, SEEK_END) != 0) {
    fclose(f);
    return false;
  }
  long size = ftell(f);
  if (size < 0) {
    fclose(f);
    return false;
  }
  rewind(f);

  uint8_t *buf = malloc(size > 0 ? (size_t)size : 1);
  if (!buf) {
    fclose(f);
    return false;
  }
  size_t n = fread(buf, 1, (size_t)size, f);
  fclose(f);

  bool ok = wp_tray_parse_icon_file(buf, n, out);
  free(buf);
  if (!ok) {
    return false;
  }

  *last_mtime = st.st_mtim;
  return true;
}

static void *icon_watcher_thread_main(void *arg) {
  (void)arg;

  char path[1024];
  if (!wp_tray_icon_path(path, sizeof(path))) {
    return NULL;
  }

  struct timespec last_mtime = {0, -1};

  wp_tray_icon_t icon;
  if (try_load_icon(path, &last_mtime, &icon)) {
    wp_tray_state_on_icon_changed(&icon);
  }

  char dir[900];
  char filename[256];
  split_dir_filename(path, dir, sizeof(dir), filename, sizeof(filename));

  int fd = inotify_init1(0);
  if (fd < 0) {
    printf("tray: inotify_init1 failed, falling back to polling for icon "
           "updates\n");
    for (;;) {
      struct timespec ts = {.tv_sec = 0, .tv_nsec = 300000000L};
      nanosleep(&ts, NULL);
      wp_tray_icon_t polled_icon;
      if (try_load_icon(path, &last_mtime, &polled_icon)) {
        wp_tray_state_on_icon_changed(&polled_icon);
      }
    }
  }

  if (inotify_add_watch(fd, dir, IN_CLOSE_WRITE | IN_MOVED_TO) < 0) {
    printf("tray: inotify_add_watch failed for %s, icon updates disabled\n",
           dir);
    close(fd);
    return NULL;
  }

  char buf[4096];
  for (;;) {
    ssize_t n = read(fd, buf, sizeof(buf));
    if (n <= 0) {
      continue;
    }
    if (!inotify_event_matches(buf, n, filename)) {
      continue;
    }

    wp_tray_icon_t new_icon;
    if (try_load_icon(path, &last_mtime, &new_icon)) {
      wp_tray_state_on_icon_changed(&new_icon);
    }
  }

  return NULL;
}

void wp_tray_files_spawn_watchers(void) {
  pthread_t menu_thread;
  if (pthread_create(&menu_thread, NULL, menu_watcher_thread_main, NULL) == 0) {
    pthread_detach(menu_thread);
  }

  pthread_t icon_thread;
  if (pthread_create(&icon_thread, NULL, icon_watcher_thread_main, NULL) == 0) {
    pthread_detach(icon_thread);
  }
}
