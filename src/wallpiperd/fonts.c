#include "fonts.h"

#include "config.h"
#include "font_rename.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#include "wallpiper/fsutil.h"

typedef struct {
  bool has_path;
  char path[768];
  uint32_t index;
} font_source_t;

typedef enum {
  GENERIC_SANS_REGULAR,
  GENERIC_SANS_BOLD,
  GENERIC_SERIF_REGULAR,
  GENERIC_MONO_REGULAR,
  GENERIC_MONO_BOLD,
} font_generic_t;

typedef struct {
  const char *filename;
  const char *family;
  const char *subfamily;
  font_generic_t generic;
} font_redirect_t;

static const font_redirect_t REDIRECTS[] = {
    {"arial.ttf", "Arial", "Regular", GENERIC_SANS_REGULAR},
    {"arialbd.ttf", "Arial", "Bold", GENERIC_SANS_BOLD},
    {"tahoma.ttf", "Tahoma", "Regular", GENERIC_SANS_REGULAR},
    {"tahomabd.ttf", "Tahoma", "Bold", GENERIC_SANS_BOLD},
    {"times.ttf", "Times New Roman", "Regular", GENERIC_SERIF_REGULAR},
    {"cour.ttf", "Courier New", "Regular", GENERIC_MONO_REGULAR},
    {"courbd.ttf", "Courier New", "Bold", GENERIC_MONO_BOLD},
};
#define REDIRECT_COUNT (sizeof(REDIRECTS) / sizeof(REDIRECTS[0]))

typedef struct {
  font_source_t sans_regular;
  font_source_t sans_bold;
  font_source_t serif_regular;
  font_source_t mono_regular;
  font_source_t mono_bold;
} font_targets_t;

static bool fc_match(const char *query, char *path_out, size_t path_out_len,
                     uint32_t *index_out) {
  int pipefd[2];
  if (pipe(pipefd) != 0) {
    return false;
  }

  pid_t pid = fork();
  if (pid < 0) {
    close(pipefd[0]);
    close(pipefd[1]);
    return false;
  }

  if (pid == 0) {
    close(pipefd[0]);
    dup2(pipefd[1], STDOUT_FILENO);
    close(pipefd[1]);
    execlp("fc-match", "fc-match", "-f", "%{file}\t%{index}\n", query,
           (char *)NULL);
    _exit(127);
  }

  close(pipefd[1]);
  char buf[2048];
  size_t total = 0;
  ssize_t n;
  while (total < sizeof(buf) - 1 &&
         (n = read(pipefd[0], buf + total, sizeof(buf) - 1 - total)) > 0) {
    total += (size_t)n;
  }
  buf[total] = '\0';
  close(pipefd[0]);

  int status = 0;
  waitpid(pid, &status, 0);
  if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
    return false;
  }

  char *tab = strchr(buf, '\t');
  if (!tab) {
    return false;
  }
  *tab = '\0';
  char *newline = strchr(tab + 1, '\n');
  if (newline) {
    *newline = '\0';
  }

  char *path = buf;
  size_t plen = strlen(path);
  while (plen > 0 && (path[plen - 1] == ' ' || path[plen - 1] == '\t')) {
    path[--plen] = '\0';
  }
  if (plen == 0) {
    return false;
  }

  int wn = snprintf(path_out, path_out_len, "%s", path);
  if (wn <= 0 || (size_t)wn >= path_out_len) {
    return false;
  }

  char *index_str = tab + 1;
  while (*index_str == ' ') {
    index_str++;
  }
  *index_out = (uint32_t)strtoul(index_str, NULL, 10);
  return true;
}

static void detect_targets(font_targets_t *t) {
  memset(t, 0, sizeof(*t));
  t->sans_regular.has_path =
      fc_match("sans-serif", t->sans_regular.path, sizeof(t->sans_regular.path),
               &t->sans_regular.index);
  t->sans_bold.has_path =
      fc_match("sans-serif:bold", t->sans_bold.path, sizeof(t->sans_bold.path),
               &t->sans_bold.index);
  t->serif_regular.has_path =
      fc_match("serif", t->serif_regular.path, sizeof(t->serif_regular.path),
               &t->serif_regular.index);
  t->mono_regular.has_path =
      fc_match("monospace", t->mono_regular.path, sizeof(t->mono_regular.path),
               &t->mono_regular.index);
  t->mono_bold.has_path =
      fc_match("monospace:bold", t->mono_bold.path, sizeof(t->mono_bold.path),
               &t->mono_bold.index);
}

static const font_source_t *targets_get(const font_targets_t *t,
                                        font_generic_t g) {
  const font_source_t *source;
  switch (g) {
  case GENERIC_SANS_REGULAR:
    source = &t->sans_regular;
    break;
  case GENERIC_SANS_BOLD:
    source = &t->sans_bold;
    break;
  case GENERIC_SERIF_REGULAR:
    source = &t->serif_regular;
    break;
  case GENERIC_MONO_REGULAR:
    source = &t->mono_regular;
    break;
  case GENERIC_MONO_BOLD:
    source = &t->mono_bold;
    break;
  default:
    return NULL;
  }
  return source->has_path ? source : NULL;
}

static bool write_renamed(const char *src_path, uint32_t face_index,
                          const char *family, const char *subfamily,
                          const char *dest_path) {
  FILE *f = fopen(src_path, "rb");
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

  uint8_t *input = malloc(size > 0 ? (size_t)size : 1);
  if (!input) {
    fclose(f);
    return false;
  }
  size_t n = fread(input, 1, (size_t)size, f);
  fclose(f);

  uint8_t *output;
  size_t output_len;
  bool ok = wp_font_rewrite_name(input, n, face_index, family, subfamily,
                                 &output, &output_len);
  free(input);
  if (!ok) {
    return false;
  }

  wp_mkdir_p_parent(dest_path);
  FILE *out_f = fopen(dest_path, "wb");
  if (!out_f) {
    free(output);
    return false;
  }
  fwrite(output, 1, output_len, out_f);
  fclose(out_f);
  free(output);
  return true;
}

bool wp_fonts_env_value(char *out, size_t out_len) {
  out[0] = '\0';

  char fonts_dir[1024];
  char cache_dir[1024];
  if (!wp_wine_fonts_dir(fonts_dir, sizeof(fonts_dir)) ||
      !wp_font_cache_dir(cache_dir, sizeof(cache_dir))) {
    return false;
  }

  font_targets_t targets;
  detect_targets(&targets);

  size_t offset = 0;
  bool first = true;

  for (size_t i = 0; i < REDIRECT_COUNT; i++) {
    const font_redirect_t *r = &REDIRECTS[i];
    const font_source_t *source = targets_get(&targets, r->generic);
    if (!source) {
      continue;
    }

    char dest[1200];
    snprintf(dest, sizeof(dest), "%s/%s", cache_dir, r->filename);

    if (!write_renamed(source->path, source->index, r->family, r->subfamily,
                       dest)) {
      printf(
          "font redirect: failed to rewrite name table for %s, skipping %s\n",
          source->path, r->filename);
      continue;
    }

    char entry[2400];
    int n = snprintf(entry, sizeof(entry), "%s%s/%s=%s", first ? "" : ";",
                     fonts_dir, r->filename, dest);
    if (n <= 0 || offset + (size_t)n >= out_len) {
      continue;
    }
    memcpy(out + offset, entry, (size_t)n);
    offset += (size_t)n;
    out[offset] = '\0';
    first = false;
  }

  return true;
}
