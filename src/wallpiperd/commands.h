#pragma once

#include <stdbool.h>
#include <stddef.h>

bool wp_commands_dispatch(const char *const *args, size_t arg_count,
                          char *err_out, size_t err_out_len);
