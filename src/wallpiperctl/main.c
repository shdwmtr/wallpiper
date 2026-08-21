#include <cJSON.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wallpiper/daemon_ctl_protocol.h"
#include "wallpiper/steam_paths.h"
#include "wallpiper/wallpaper_catalog.h"

typedef struct {
  char *data;
  size_t len;
  size_t cap;
} strbuf_t;

static void strbuf_init(strbuf_t *sb) {
  sb->cap = 256;
  sb->data = malloc(sb->cap);
  sb->len = 0;
  sb->data[0] = '\0';
}

static void strbuf_append_n(strbuf_t *sb, const char *s, size_t n) {
  if (sb->len + n + 1 > sb->cap) {
    while (sb->len + n + 1 > sb->cap) {
      sb->cap *= 2;
    }
    sb->data = realloc(sb->data, sb->cap);
  }
  memcpy(sb->data + sb->len, s, n);
  sb->len += n;
  sb->data[sb->len] = '\0';
}

static void strbuf_append(strbuf_t *sb, const char *s) {
  strbuf_append_n(sb, s, strlen(s));
}

static void strbuf_indent(strbuf_t *sb, int depth) {
  for (int i = 0; i < depth; i++) {
    strbuf_append(sb, "  ");
  }
}

static void append_json_string(strbuf_t *sb, const char *s) {
  strbuf_append(sb, "\"");
  for (const unsigned char *p = (const unsigned char *)s; *p; p++) {
    switch (*p) {
    case '"':
      strbuf_append(sb, "\\\"");
      break;
    case '\\':
      strbuf_append(sb, "\\\\");
      break;
    case '\n':
      strbuf_append(sb, "\\n");
      break;
    case '\r':
      strbuf_append(sb, "\\r");
      break;
    case '\t':
      strbuf_append(sb, "\\t");
      break;
    default:
      if (*p < 0x20) {
        char esc[8];
        snprintf(esc, sizeof(esc), "\\u%04x", *p);
        strbuf_append(sb, esc);
      } else {
        char c = (char)*p;
        strbuf_append_n(sb, &c, 1);
      }
    }
  }
  strbuf_append(sb, "\"");
}

static void append_json_number(strbuf_t *sb, double value) {
  char buf[64];
  if (value == (double)(long long)value && fabs(value) < 1e15) {
    snprintf(buf, sizeof(buf), "%lld", (long long)value);
  } else {
    snprintf(buf, sizeof(buf), "%g", value);
  }
  strbuf_append(sb, buf);
}

static int compare_cjson_keys(const void *a, const void *b) {
  const cJSON *const *ja = a;
  const cJSON *const *jb = b;
  return strcmp((*ja)->string, (*jb)->string);
}

static void append_json_value_sorted(strbuf_t *sb, const cJSON *node,
                                     int depth) {
  if (cJSON_IsNull(node) || node == NULL) {
    strbuf_append(sb, "null");
  } else if (cJSON_IsBool(node)) {
    strbuf_append(sb, cJSON_IsTrue(node) ? "true" : "false");
  } else if (cJSON_IsNumber(node)) {
    append_json_number(sb, node->valuedouble);
  } else if (cJSON_IsString(node)) {
    append_json_string(sb, node->valuestring);
  } else if (cJSON_IsArray(node)) {
    if (!node->child) {
      strbuf_append(sb, "[]");
      return;
    }
    strbuf_append(sb, "[\n");
    bool first = true;
    for (const cJSON *item = node->child; item; item = item->next) {
      if (!first) {
        strbuf_append(sb, ",\n");
      }
      first = false;
      strbuf_indent(sb, depth + 1);
      append_json_value_sorted(sb, item, depth + 1);
    }
    strbuf_append(sb, "\n");
    strbuf_indent(sb, depth);
    strbuf_append(sb, "]");
  } else if (cJSON_IsObject(node)) {
    if (!node->child) {
      strbuf_append(sb, "{}");
      return;
    }
    size_t count = 0;
    for (const cJSON *item = node->child; item; item = item->next) {
      count++;
    }
    const cJSON **items = malloc(sizeof(cJSON *) * count);
    size_t i = 0;
    for (const cJSON *item = node->child; item; item = item->next) {
      items[i++] = item;
    }
    qsort(items, count, sizeof(cJSON *), compare_cjson_keys);

    strbuf_append(sb, "{\n");
    for (i = 0; i < count; i++) {
      if (i > 0) {
        strbuf_append(sb, ",\n");
      }
      strbuf_indent(sb, depth + 1);
      append_json_string(sb, items[i]->string);
      strbuf_append(sb, ": ");
      append_json_value_sorted(sb, items[i], depth + 1);
    }
    strbuf_append(sb, "\n");
    strbuf_indent(sb, depth);
    strbuf_append(sb, "}");
    free(items);
  } else {
    strbuf_append(sb, "null");
  }
}

static void append_json_passthrough(strbuf_t *sb, const char *raw_json,
                                    int depth) {
  cJSON *value = cJSON_Parse(raw_json);
  if (!value) {
    strbuf_append(sb, "null");
    return;
  }
  append_json_value_sorted(sb, value, depth);
  cJSON_Delete(value);
}

static const char *const DAEMON_COMMANDS[] = {
    "pause", "resume", "debug", "nodebug", "set", "windowbrowser", "inject",
};
#define DAEMON_COMMAND_COUNT                                                   \
  (sizeof(DAEMON_COMMANDS) / sizeof(DAEMON_COMMANDS[0]))

static bool is_daemon_command(const char *cmd) {
  for (size_t i = 0; i < DAEMON_COMMAND_COUNT; i++) {
    if (strcmp(cmd, DAEMON_COMMANDS[i]) == 0) {
      return true;
    }
  }
  return false;
}

static void print_usage(void) {
  fprintf(stderr, "usage: wallpiperctl <command> [args]\n"
                  "\n"
                  "daemon commands (require a running wallpiper-daemon):\n"
                  "  pause | resume | debug | nodebug\n"
                  "  set <file> [location]\n"
                  "  set --id <workshop_id> [location]\n"
                  "  windowbrowser\n"
                  "  inject <file> [location]\n"
                  "\n"
                  "standalone commands:\n"
                  "  list-wallpapers [-j]\n"
                  "  list-properties <workshop_id> [-j]\n"
                  "  check-config\n");
}

static bool has_flag(int argc, char **argv, const char *flag) {
  for (int i = 0; i < argc; i++) {
    if (strcmp(argv[i], flag) == 0) {
      return true;
    }
  }
  return false;
}

static bool list_wallpapers_cmd(int argc, char **argv, char *err_out,
                                size_t err_out_len) {
  bool json = has_flag(argc, argv, "-j");

  char workshop_dir[1024];
  if (!wp_workshop_content_dir(workshop_dir, sizeof(workshop_dir), err_out,
                               err_out_len)) {
    return false;
  }

  const size_t max_wallpapers = 8192;
  wp_wallpaper_info_t *infos =
      calloc(max_wallpapers, sizeof(wp_wallpaper_info_t));
  if (!infos) {
    snprintf(err_out, err_out_len, "out of memory");
    return false;
  }

  size_t count = 0;
  if (!wp_wallpaper_catalog_list(workshop_dir, infos, max_wallpapers, &count)) {
    snprintf(err_out, err_out_len,
             "could not read workshop content directory: %.400s", workshop_dir);
    free(infos);
    return false;
  }

  if (json) {
    strbuf_t sb;
    strbuf_init(&sb);
    if (count == 0) {
      strbuf_append(&sb, "[]");
    } else {
      strbuf_append(&sb, "[\n");
      for (size_t i = 0; i < count; i++) {
        if (i > 0) {
          strbuf_append(&sb, ",\n");
        }
        strbuf_indent(&sb, 1);
        strbuf_append(&sb, "{\n");
        strbuf_indent(&sb, 2);
        strbuf_append(&sb, "\"id\": ");
        append_json_string(&sb, infos[i].id);
        strbuf_append(&sb, ",\n");
        strbuf_indent(&sb, 2);
        strbuf_append(&sb, "\"title\": ");
        append_json_string(&sb, infos[i].title);
        strbuf_append(&sb, ",\n");
        strbuf_indent(&sb, 2);
        strbuf_append(&sb, "\"kind\": ");
        append_json_string(&sb, infos[i].kind);
        strbuf_append(&sb, "\n");
        strbuf_indent(&sb, 1);
        strbuf_append(&sb, "}");
      }
      strbuf_append(&sb, "\n]");
    }
    printf("%s\n", sb.data);
    free(sb.data);
    free(infos);
    return true;
  }

  if (count == 0) {
    printf("no workshop wallpapers found\n");
  } else {
    for (size_t i = 0; i < count; i++) {
      printf("%s  %s  (%s)\n", infos[i].id, infos[i].title, infos[i].kind);
    }
  }
  free(infos);
  return true;
}

static bool list_properties_cmd(int argc, char **argv, char *err_out,
                                size_t err_out_len) {
  bool json = has_flag(argc, argv, "-j");

  const char *id = NULL;
  for (int i = 0; i < argc; i++) {
    if (strcmp(argv[i], "-j") != 0) {
      id = argv[i];
      break;
    }
  }
  if (!id) {
    snprintf(err_out, err_out_len, "usage: list-properties <workshop_id> [-j]");
    return false;
  }

  char workshop_dir[1024];
  if (!wp_workshop_content_dir(workshop_dir, sizeof(workshop_dir), err_out,
                               err_out_len)) {
    return false;
  }

  const size_t max_properties = 1024;
  wp_property_info_t *props =
      calloc(max_properties, sizeof(wp_property_info_t));
  if (!props) {
    snprintf(err_out, err_out_len, "out of memory");
    return false;
  }

  if (!wp_wallpaper_catalog_resolve(workshop_dir, id, err_out, err_out_len)) {
    free(props);
    return false;
  }

  char title[256];
  size_t count = 0;
  if (!wp_wallpaper_catalog_properties(workshop_dir, id, title, sizeof(title),
                                       props, max_properties, &count)) {
    snprintf(err_out, err_out_len, "could not read properties for id %s", id);
    free(props);
    return false;
  }

  if (json) {
    strbuf_t sb;
    strbuf_init(&sb);
    strbuf_append(&sb, "{\n");
    strbuf_indent(&sb, 1);
    strbuf_append(&sb, "\"id\": ");
    append_json_string(&sb, id);
    strbuf_append(&sb, ",\n");
    strbuf_indent(&sb, 1);
    strbuf_append(&sb, "\"properties\": ");
    if (count == 0) {
      strbuf_append(&sb, "[]");
    } else {
      strbuf_append(&sb, "[\n");
      for (size_t i = 0; i < count; i++) {
        if (i > 0) {
          strbuf_append(&sb, ",\n");
        }
        strbuf_indent(&sb, 2);
        strbuf_append(&sb, "{\n");
        strbuf_indent(&sb, 3);
        strbuf_append(&sb, "\"key\": ");
        append_json_string(&sb, props[i].key);
        strbuf_append(&sb, ",\n");
        strbuf_indent(&sb, 3);
        strbuf_append(&sb, "\"kind\": ");
        append_json_string(&sb, props[i].kind);
        strbuf_append(&sb, ",\n");
        strbuf_indent(&sb, 3);
        strbuf_append(&sb, "\"text\": ");
        append_json_string(&sb, props[i].text);
        strbuf_append(&sb, ",\n");
        strbuf_indent(&sb, 3);
        strbuf_append(&sb, "\"value\": ");
        append_json_passthrough(&sb, props[i].value_json, 3);
        strbuf_append(&sb, "\n");
        strbuf_indent(&sb, 2);
        strbuf_append(&sb, "}");
      }
      strbuf_append(&sb, "\n");
      strbuf_indent(&sb, 1);
      strbuf_append(&sb, "]");
    }
    strbuf_append(&sb, ",\n");
    strbuf_indent(&sb, 1);
    strbuf_append(&sb, "\"title\": ");
    append_json_string(&sb, title);
    strbuf_append(&sb, "\n}");
    printf("%s\n", sb.data);
    free(sb.data);
    free(props);
    return true;
  }

  printf("%s (%s)\n", title, id);
  if (count == 0) {
    printf("  no properties\n");
  } else {
    for (size_t i = 0; i < count; i++) {
      printf("  %-24s %-8s %-32s = %s\n", props[i].key, props[i].kind,
             props[i].text, props[i].value_json);
    }
  }
  free(props);
  return true;
}

int main(int argc, char **argv) {
  if (argc < 2) {
    print_usage();
    return 1;
  }

  const char *cmd = argv[1];
  char err[512];
  bool ok;

  if (strcmp(cmd, "list-wallpapers") == 0) {
    ok = list_wallpapers_cmd(argc - 2, argv + 2, err, sizeof(err));
  } else if (strcmp(cmd, "list-properties") == 0) {
    ok = list_properties_cmd(argc - 2, argv + 2, err, sizeof(err));
  } else if (strcmp(cmd, "check-config") == 0) {
    wp_describe();
    ok = true;
  } else if (is_daemon_command(cmd)) {
    ok = wp_send_daemon_command((const char *const *)(argv + 1),
                                (size_t)(argc - 1), err, sizeof(err));
  } else {
    snprintf(err, sizeof(err), "unknown command: %s", cmd);
    ok = false;
  }

  if (!ok) {
    fprintf(stderr, "wallpiperctl: %s\n", err);
    return 1;
  }
  return 0;
}
