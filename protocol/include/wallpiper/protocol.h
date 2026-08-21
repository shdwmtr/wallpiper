#pragma once

#include <stdbool.h>
#include <stddef.h>

#define WALLPIPER_CAPTURE_SOCKET_PATH "/tmp/wallpiper-capture.sock"

bool wp_ctl_socket_path(const char *portal_name, char *out, size_t out_len);
bool wp_daemon_ctl_socket_path(char *out, size_t out_len);
bool wp_temp_dir(char *out, size_t out_len);
bool wp_runtime_dir(char *out, size_t out_len);
bool wp_windowbrowser_socket_path(char *out, size_t out_len);
bool wp_inject_socket_path(char *out, size_t out_len);
