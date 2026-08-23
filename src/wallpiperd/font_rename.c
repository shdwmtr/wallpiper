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

#include "font_rename.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>

#define HEAD_CHECKSUM_MAGIC 0xB1B0AFBAu

typedef struct {
  uint8_t *data;
  size_t len;
  size_t cap;
} bytebuf_t;

static void bytebuf_init(bytebuf_t *b) {
  b->data = NULL;
  b->len = 0;
  b->cap = 0;
}

static void bytebuf_reserve(bytebuf_t *b, size_t extra) {
  if (b->len + extra <= b->cap) {
    return;
  }
  size_t new_cap = b->cap == 0 ? 256 : b->cap * 2;
  while (new_cap < b->len + extra) {
    new_cap *= 2;
  }
  b->data = realloc(b->data, new_cap);
  b->cap = new_cap;
}

static void bytebuf_append(bytebuf_t *b, const uint8_t *bytes, size_t n) {
  bytebuf_reserve(b, n);
  memcpy(b->data + b->len, bytes, n);
  b->len += n;
}

static void bytebuf_append_u16be(bytebuf_t *b, uint16_t v) {
  uint8_t bytes[2] = {(uint8_t)(v >> 8), (uint8_t)v};
  bytebuf_append(b, bytes, 2);
}

static void bytebuf_append_u32be(bytebuf_t *b, uint32_t v) {
  uint8_t bytes[4] = {(uint8_t)(v >> 24), (uint8_t)(v >> 16), (uint8_t)(v >> 8),
                      (uint8_t)v};
  bytebuf_append(b, bytes, 4);
}

static void bytebuf_append_str(bytebuf_t *b, const char *s) {
  bytebuf_append(b, (const uint8_t *)s, strlen(s));
}

static void bytebuf_append_utf16be(bytebuf_t *b, const char *s) {
  const unsigned char *p = (const unsigned char *)s;
  while (*p) {
    uint32_t cp;
    int extra;
    if (*p < 0x80) {
      cp = *p;
      extra = 0;
    } else if ((*p & 0xE0) == 0xC0) {
      cp = *p & 0x1F;
      extra = 1;
    } else if ((*p & 0xF0) == 0xE0) {
      cp = *p & 0x0F;
      extra = 2;
    } else if ((*p & 0xF8) == 0xF0) {
      cp = *p & 0x07;
      extra = 3;
    } else {
      p++;
      continue;
    }
    const unsigned char *start = p;
    p++;
    bool valid = true;
    for (int i = 0; i < extra; i++) {
      if ((*p & 0xC0) != 0x80) {
        valid = false;
        break;
      }
      cp = (cp << 6) | (*p & 0x3F);
      p++;
    }
    if (!valid) {
      p = start + 1;
      continue;
    }
    if (cp <= 0xFFFF) {
      bytebuf_append_u16be(b, (uint16_t)cp);
    } else {
      cp -= 0x10000;
      bytebuf_append_u16be(b, (uint16_t)(0xD800 + (cp >> 10)));
      bytebuf_append_u16be(b, (uint16_t)(0xDC00 + (cp & 0x3FF)));
    }
  }
}

static bool read_u16(const uint8_t *buf, size_t buf_len, size_t off,
                     uint16_t *out) {
  if (off + 2 > buf_len) {
    return false;
  }
  *out = (uint16_t)((buf[off] << 8) | buf[off + 1]);
  return true;
}

static bool read_u32(const uint8_t *buf, size_t buf_len, size_t off,
                     uint32_t *out) {
  if (off + 4 > buf_len) {
    return false;
  }
  *out = ((uint32_t)buf[off] << 24) | ((uint32_t)buf[off + 1] << 16) |
         ((uint32_t)buf[off + 2] << 8) | buf[off + 3];
  return true;
}

static bool face_offset(const uint8_t *input, size_t input_len,
                        uint32_t face_index, size_t *out) {
  if (input_len >= 12 && memcmp(input, "ttcf", 4) == 0) {
    uint32_t num_fonts;
    if (!read_u32(input, input_len, 8, &num_fonts) || face_index >= num_fonts) {
      return false;
    }
    size_t entry_off = 12 + (size_t)face_index * 4;
    uint32_t offset;
    if (!read_u32(input, input_len, entry_off, &offset)) {
      return false;
    }
    *out = offset;
    return true;
  }
  *out = 0;
  return true;
}

typedef struct {
  uint8_t tag[4];
  uint8_t *data;
  size_t data_len;
} font_table_t;

static void free_tables(font_table_t *tables, size_t count) {
  for (size_t i = 0; i < count; i++) {
    free(tables[i].data);
  }
  free(tables);
}

static bool read_tables(const uint8_t *input, size_t input_len,
                        size_t sfnt_offset, uint32_t *version,
                        font_table_t **out_tables, size_t *out_count) {
  uint32_t v;
  uint16_t num_tables;
  if (!read_u32(input, input_len, sfnt_offset, &v) ||
      !read_u16(input, input_len, sfnt_offset + 4, &num_tables)) {
    return false;
  }

  font_table_t *tables = calloc(num_tables, sizeof(font_table_t));
  if (num_tables > 0 && !tables) {
    return false;
  }

  for (uint16_t i = 0; i < num_tables; i++) {
    size_t entry_off = sfnt_offset + 12 + (size_t)i * 16;
    if (entry_off + 16 > input_len) {
      free_tables(tables, i);
      return false;
    }
    memcpy(tables[i].tag, input + entry_off, 4);

    uint32_t offset, length;
    if (!read_u32(input, input_len, entry_off + 8, &offset) ||
        !read_u32(input, input_len, entry_off + 12, &length)) {
      free_tables(tables, i);
      return false;
    }
    if ((size_t)offset + length > input_len) {
      free_tables(tables, i);
      return false;
    }

    tables[i].data = malloc(length > 0 ? length : 1);
    if (!tables[i].data) {
      free_tables(tables, i);
      return false;
    }
    memcpy(tables[i].data, input + offset, length);
    tables[i].data_len = length;
  }

  *version = v;
  *out_tables = tables;
  *out_count = num_tables;
  return true;
}

static void strip_spaces(const char *s, char *out, size_t out_len) {
  size_t j = 0;
  for (size_t i = 0; s[i] != '\0' && j + 1 < out_len; i++) {
    if (s[i] != ' ') {
      out[j++] = s[i];
    }
  }
  out[j] = '\0';
}

static void build_name_table(const char *family, const char *subfamily,
                             bytebuf_t *out) {
  char full_name[512];
  if (strcasecmp(subfamily, "regular") == 0) {
    snprintf(full_name, sizeof(full_name), "%s", family);
  } else {
    snprintf(full_name, sizeof(full_name), "%s %s", family, subfamily);
  }

  char family_nospace[256];
  char subfamily_nospace[256];
  strip_spaces(family, family_nospace, sizeof(family_nospace));
  strip_spaces(subfamily, subfamily_nospace, sizeof(subfamily_nospace));

  char postscript_name[512];
  snprintf(postscript_name, sizeof(postscript_name), "%s-%s", family_nospace,
           subfamily_nospace);

  char unique_id[600];
  snprintf(unique_id, sizeof(unique_id), "1.000;WLPR;%s", postscript_name);

  const char *entry_values[5] = {family, subfamily, unique_id, full_name,
                                 postscript_name};
  const uint16_t entry_ids[5] = {1, 2, 3, 4, 6};

  const uint16_t platform_ids[2] = {3, 1};
  const uint16_t encoding_ids[2] = {1, 0};
  const uint16_t language_ids[2] = {0x0409, 0};

  typedef struct {
    uint16_t platform_id, encoding_id, language_id, name_id, length, offset;
  } record_t;
  record_t records[10];
  int record_count = 0;

  bytebuf_t storage;
  bytebuf_init(&storage);

  for (int pi = 0; pi < 2; pi++) {
    for (int ei = 0; ei < 5; ei++) {
      size_t before = storage.len;
      if (platform_ids[pi] == 3) {
        bytebuf_append_utf16be(&storage, entry_values[ei]);
      } else {
        bytebuf_append_str(&storage, entry_values[ei]);
      }
      size_t after = storage.len;

      record_t r = {
          platform_ids[pi], encoding_ids[pi],           language_ids[pi],
          entry_ids[ei],    (uint16_t)(after - before), (uint16_t)before,
      };
      records[record_count++] = r;
    }
  }

  uint16_t count = (uint16_t)record_count;
  uint16_t storage_offset = (uint16_t)(6 + count * 12);

  bytebuf_append_u16be(out, 0);
  bytebuf_append_u16be(out, count);
  bytebuf_append_u16be(out, storage_offset);
  for (int i = 0; i < record_count; i++) {
    bytebuf_append_u16be(out, records[i].platform_id);
    bytebuf_append_u16be(out, records[i].encoding_id);
    bytebuf_append_u16be(out, records[i].language_id);
    bytebuf_append_u16be(out, records[i].name_id);
    bytebuf_append_u16be(out, records[i].length);
    bytebuf_append_u16be(out, records[i].offset);
  }
  bytebuf_append(out, storage.data, storage.len);
  free(storage.data);
}

static uint32_t table_checksum(const uint8_t *data, size_t len) {
  uint32_t sum = 0;
  for (size_t i = 0; i < len; i += 4) {
    uint32_t word = 0;
    for (int b = 0; b < 4; b++) {
      uint8_t byte = (i + (size_t)b < len) ? data[i + (size_t)b] : 0;
      word = (word << 8) | byte;
    }
    sum += word;
  }
  return sum;
}

static int compare_tags(const void *a, const void *b) {
  const font_table_t *ta = a;
  const font_table_t *tb = b;
  return memcmp(ta->tag, tb->tag, 4);
}

static bool serialize(uint32_t version, font_table_t *tables, size_t count,
                      uint8_t **out, size_t *out_len) {
  for (size_t i = 0; i < count; i++) {
    if (memcmp(tables[i].tag, "head", 4) == 0 && tables[i].data_len >= 12) {
      memset(tables[i].data + 8, 0, 4);
    }
  }

  qsort(tables, count, sizeof(font_table_t), compare_tags);

  uint16_t num_tables = (uint16_t)count;
  uint16_t entry_selector = 0;
  while ((1u << (entry_selector + 1)) <= num_tables) {
    entry_selector++;
  }
  uint16_t search_range = (uint16_t)((1u << entry_selector) * 16);
  uint16_t range_shift = (uint16_t)(num_tables * 16 - search_range);

  size_t header_size = 12 + count * 16;

  typedef struct {
    uint8_t tag[4];
    uint32_t checksum;
    uint32_t offset;
    uint32_t length;
  } dir_entry_t;
  dir_entry_t *directory = malloc(count > 0 ? count * sizeof(dir_entry_t) : 1);
  if (count > 0 && !directory) {
    return false;
  }

  bytebuf_t body;
  bytebuf_init(&body);
  long head_checksum_pos = -1;

  for (size_t i = 0; i < count; i++) {
    while (body.len % 4 != 0) {
      uint8_t z = 0;
      bytebuf_append(&body, &z, 1);
    }
    size_t offset = header_size + body.len;
    uint32_t checksum = table_checksum(tables[i].data, tables[i].data_len);

    memcpy(directory[i].tag, tables[i].tag, 4);
    directory[i].checksum = checksum;
    directory[i].offset = (uint32_t)offset;
    directory[i].length = (uint32_t)tables[i].data_len;

    if (memcmp(tables[i].tag, "head", 4) == 0) {
      head_checksum_pos = (long)(offset + 8);
    }
    bytebuf_append(&body, tables[i].data, tables[i].data_len);
  }
  while (body.len % 4 != 0) {
    uint8_t z = 0;
    bytebuf_append(&body, &z, 1);
  }

  bytebuf_t result;
  bytebuf_init(&result);
  bytebuf_append_u32be(&result, version);
  bytebuf_append_u16be(&result, num_tables);
  bytebuf_append_u16be(&result, search_range);
  bytebuf_append_u16be(&result, entry_selector);
  bytebuf_append_u16be(&result, range_shift);
  for (size_t i = 0; i < count; i++) {
    bytebuf_append(&result, directory[i].tag, 4);
    bytebuf_append_u32be(&result, directory[i].checksum);
    bytebuf_append_u32be(&result, directory[i].offset);
    bytebuf_append_u32be(&result, directory[i].length);
  }
  bytebuf_append(&result, body.data, body.len);
  free(body.data);
  free(directory);

  if (head_checksum_pos >= 0) {
    uint32_t file_checksum = table_checksum(result.data, result.len);
    uint32_t adjustment = HEAD_CHECKSUM_MAGIC - file_checksum;
    size_t pos = (size_t)head_checksum_pos;
    result.data[pos] = (uint8_t)(adjustment >> 24);
    result.data[pos + 1] = (uint8_t)(adjustment >> 16);
    result.data[pos + 2] = (uint8_t)(adjustment >> 8);
    result.data[pos + 3] = (uint8_t)adjustment;
  }

  *out = result.data;
  *out_len = result.len;
  return true;
}

bool wp_font_rewrite_name(const uint8_t *input, size_t input_len,
                          uint32_t face_index, const char *family,
                          const char *subfamily, uint8_t **out,
                          size_t *out_len) {
  size_t sfnt_offset;
  if (!face_offset(input, input_len, face_index, &sfnt_offset)) {
    return false;
  }

  uint32_t version;
  font_table_t *tables;
  size_t count;
  if (!read_tables(input, input_len, sfnt_offset, &version, &tables, &count)) {
    return false;
  }

  bytebuf_t name_table;
  bytebuf_init(&name_table);
  build_name_table(family, subfamily, &name_table);

  bool found = false;
  for (size_t i = 0; i < count; i++) {
    if (memcmp(tables[i].tag, "name", 4) == 0) {
      free(tables[i].data);
      tables[i].data = name_table.data;
      tables[i].data_len = name_table.len;
      found = true;
      break;
    }
  }
  if (!found) {
    font_table_t *new_tables =
        realloc(tables, (count + 1) * sizeof(font_table_t));
    if (!new_tables) {
      free(name_table.data);
      free_tables(tables, count);
      return false;
    }
    tables = new_tables;
    memcpy(tables[count].tag, "name", 4);
    tables[count].data = name_table.data;
    tables[count].data_len = name_table.len;
    count++;
  }

  bool ok = serialize(version, tables, count, out, out_len);

  free_tables(tables, count);
  return ok;
}
