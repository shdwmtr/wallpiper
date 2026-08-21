#pragma once

#include <stdbool.h>
#include <stdint.h>

void wp_capture_log(const char *fmt, ...);
bool wp_capture_should_sample(uint64_t count);

#define WP_LOG(...) wp_capture_log(__VA_ARGS__)
