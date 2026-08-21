#pragma once

#include <stdbool.h>
#include <stdint.h>

#define WP_CAPTURE_MAX_FDS 2

typedef enum {
  WP_CAPTURE_EVENT_BUF,
  WP_CAPTURE_EVENT_FRAME,
  WP_CAPTURE_EVENT_SHM,
} wp_capture_event_tag_t;

typedef struct {
  wp_capture_event_tag_t tag;
  uint32_t slot;
  uint32_t width;
  uint32_t height;
  uint32_t stride;
  uint64_t modifier;
  int fds[WP_CAPTURE_MAX_FDS];
  int nfds;
} wp_capture_event_t;

int wp_bind_capture_socket(void);
bool wp_recv_capture_event(int sock_fd, wp_capture_event_t *out);

typedef struct wp_capture_link wp_capture_link_t;

wp_capture_link_t *wp_capture_link_create(void);
void wp_capture_link_destroy(wp_capture_link_t *link);

bool wp_capture_link_send_buf(wp_capture_link_t *link, uint32_t slot,
                              uint32_t width, uint32_t height,
                              uint32_t format_raw, uint32_t stride,
                              uint64_t modifier, int image_fd, int sync_fd);
bool wp_capture_link_send_frame(wp_capture_link_t *link, uint32_t slot,
                                int sync_fd);
