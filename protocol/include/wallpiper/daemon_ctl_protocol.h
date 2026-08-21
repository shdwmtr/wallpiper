#pragma once

#include <stdbool.h>
#include <stddef.h>

bool wp_daemon_ctl_encode_ok(char *out, size_t out_len);
bool wp_daemon_ctl_encode_err(const char *message, char *out, size_t out_len);
bool wp_daemon_ctl_parse_response(const char *line, bool *ok_out, char *err_out,
                                  size_t err_out_len);

bool wp_send_daemon_command(const char *const *args, size_t arg_count,
                            char *err_out, size_t err_out_len);
