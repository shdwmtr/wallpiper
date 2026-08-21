#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

bool wp_font_rewrite_name(const uint8_t *input, size_t input_len,
                          uint32_t face_index, const char *family,
                          const char *subfamily, uint8_t **out,
                          size_t *out_len);
