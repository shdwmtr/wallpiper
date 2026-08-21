#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "wallpiper/monitor_geometry.h"

typedef enum {
  WP_CTL_REQUEST_GEOMETRY,
  WP_CTL_REQUEST_DETACH,
  WP_CTL_REQUEST_DEBUG_ON,
  WP_CTL_REQUEST_DEBUG_OFF,
  WP_CTL_REQUEST_CURSOR_POS,
} wp_ctl_request_t;

typedef enum {
  WP_CTL_RESPONSE_OK,
  WP_CTL_RESPONSE_ERR,
  WP_CTL_RESPONSE_GEOMETRY,
  WP_CTL_RESPONSE_CURSOR_POS,
} wp_ctl_response_tag_t;

typedef struct {
  wp_ctl_response_tag_t tag;
  char err[256];
  wp_monitor_geometry_t geometry;
  int32_t cursor_x;
  int32_t cursor_y;
} wp_ctl_response_t;

bool wp_ctl_request_encode(wp_ctl_request_t request, char *out, size_t out_len);
bool wp_ctl_request_parse(const char *line, wp_ctl_request_t *out);

bool wp_ctl_response_encode(const wp_ctl_response_t *response, char *out,
                            size_t out_len);
bool wp_ctl_response_parse(const char *line, wp_ctl_response_t *out);

bool wp_send_ctl_request(const char *portal_name, wp_ctl_request_t request,
                         wp_ctl_response_t *out);

typedef struct wp_ctl_listener wp_ctl_listener_t;
typedef void (*wp_ctl_cursor_pos_fn)(void *ctx, wp_ctl_response_t *out);

wp_ctl_listener_t *wp_ctl_listener_start(const char *portal_name,
                                         wp_ctl_cursor_pos_fn cursor_fn,
                                         void *cursor_ctx);
void wp_ctl_listener_stop(wp_ctl_listener_t *listener);

bool wp_ctl_listener_poll(wp_ctl_listener_t *listener,
                          wp_ctl_request_t *out_request);
void wp_ctl_listener_reply(wp_ctl_listener_t *listener,
                           const wp_ctl_response_t *response);
