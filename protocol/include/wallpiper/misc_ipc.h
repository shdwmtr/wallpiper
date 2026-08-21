#pragma once

#include <stdbool.h>
#include <stddef.h>

bool wp_send_windowbrowser_trigger(const char *const *keysyms,
                                   size_t keysym_count, char *err_out,
                                   size_t err_out_len);

bool wp_send_inject_select(const char *unix_path, const char *location,
                           char *reply_out, size_t reply_out_len, char *err_out,
                           size_t err_out_len);
