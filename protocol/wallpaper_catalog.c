#include "wallpiper/wallpaper_catalog.h"

#include <cJSON.h>
#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/stat.h>

static char *read_file(const char *path) {
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

static bool project_json_path(const char *workshop_content_dir, const char *id,
                              char *out, size_t out_len) {
  int n =
      snprintf(out, out_len, "%s/%s/project.json", workshop_content_dir, id);
  return n > 0 && (size_t)n < out_len;
}

static int compare_wallpaper_info(const void *a, const void *b) {
  const wp_wallpaper_info_t *wa = a;
  const wp_wallpaper_info_t *wb = b;
  return strcasecmp(wa->title, wb->title);
}

bool wp_wallpaper_catalog_list(const char *workshop_content_dir,
                               wp_wallpaper_info_t *out, size_t max_out,
                               size_t *out_count) {
  *out_count = 0;

  DIR *dir = opendir(workshop_content_dir);
  if (!dir) {
    return false;
  }

  struct dirent *entry;
  while ((entry = readdir(dir)) != NULL) {
    if (entry->d_name[0] == '.') {
      continue;
    }
    if (*out_count >= max_out) {
      break;
    }

    char path[1024];
    if (!project_json_path(workshop_content_dir, entry->d_name, path,
                           sizeof(path))) {
      continue;
    }

    char *json_text = read_file(path);
    if (!json_text) {
      continue;
    }

    cJSON *root = cJSON_Parse(json_text);
    free(json_text);
    if (!root) {
      continue;
    }

    wp_wallpaper_info_t *info = &out[*out_count];
    memset(info, 0, sizeof(*info));
    snprintf(info->id, sizeof(info->id), "%.*s", (int)sizeof(info->id) - 1,
             entry->d_name);

    cJSON *title = cJSON_GetObjectItemCaseSensitive(root, "title");
    snprintf(info->title, sizeof(info->title), "%s",
             cJSON_IsString(title) ? title->valuestring : "(untitled)");

    cJSON *type = cJSON_GetObjectItemCaseSensitive(root, "type");
    snprintf(info->kind, sizeof(info->kind), "%s",
             cJSON_IsString(type) ? type->valuestring : "unknown");

    cJSON_Delete(root);
    (*out_count)++;
  }
  closedir(dir);

  qsort(out, *out_count, sizeof(wp_wallpaper_info_t), compare_wallpaper_info);
  return true;
}

static int compare_property_info(const void *a, const void *b) {
  const wp_property_info_t *pa = a;
  const wp_property_info_t *pb = b;
  return strcmp(pa->key, pb->key);
}

bool wp_wallpaper_catalog_properties(const char *workshop_content_dir,
                                     const char *id, char *title_out,
                                     size_t title_out_len,
                                     wp_property_info_t *out, size_t max_out,
                                     size_t *out_count) {
  *out_count = 0;
  if (title_out && title_out_len > 0) {
    title_out[0] = '\0';
  }

  char path[1024];
  if (!project_json_path(workshop_content_dir, id, path, sizeof(path))) {
    return false;
  }

  char *json_text = read_file(path);
  if (!json_text) {
    return false;
  }

  cJSON *root = cJSON_Parse(json_text);
  free(json_text);
  if (!root) {
    return false;
  }

  cJSON *title = cJSON_GetObjectItemCaseSensitive(root, "title");
  if (title_out) {
    snprintf(title_out, title_out_len, "%s",
             cJSON_IsString(title) ? title->valuestring : "(untitled)");
  }

  cJSON *general = cJSON_GetObjectItemCaseSensitive(root, "general");
  cJSON *properties =
      cJSON_IsObject(general)
          ? cJSON_GetObjectItemCaseSensitive(general, "properties")
          : NULL;

  if (cJSON_IsObject(properties)) {
    cJSON *prop = NULL;
    cJSON_ArrayForEach(prop, properties) {
      if (*out_count >= max_out) {
        break;
      }

      wp_property_info_t *info = &out[*out_count];
      memset(info, 0, sizeof(*info));
      snprintf(info->key, sizeof(info->key), "%s",
               prop->string ? prop->string : "");

      cJSON *kind = cJSON_GetObjectItemCaseSensitive(prop, "type");
      snprintf(info->kind, sizeof(info->kind), "%s",
               cJSON_IsString(kind) ? kind->valuestring : "unknown");

      cJSON *text = cJSON_GetObjectItemCaseSensitive(prop, "text");
      snprintf(info->text, sizeof(info->text), "%s",
               cJSON_IsString(text) ? text->valuestring : "");

      cJSON *value = cJSON_GetObjectItemCaseSensitive(prop, "value");
      char *value_str = value ? cJSON_PrintUnformatted(value) : NULL;
      snprintf(info->value_json, sizeof(info->value_json), "%s",
               value_str ? value_str : "null");
      free(value_str);

      (*out_count)++;
    }
  }

  cJSON_Delete(root);

  qsort(out, *out_count, sizeof(wp_property_info_t), compare_property_info);
  return true;
}

bool wp_wallpaper_catalog_resolve(const char *workshop_content_dir,
                                  const char *id, char *err_out,
                                  size_t err_out_len) {
  char path[1024];
  if (!project_json_path(workshop_content_dir, id, path, sizeof(path))) {
    snprintf(err_out, err_out_len, "workshop id too long");
    return false;
  }

  struct stat st;
  if (stat(path, &st) != 0 || !S_ISREG(st.st_mode)) {
    snprintf(err_out, err_out_len,
             "no workshop wallpaper found for id %s: expected %s", id, path);
    return false;
  }
  return true;
}
