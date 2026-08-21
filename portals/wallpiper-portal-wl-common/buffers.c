#include "state_internal.h"

#include <stdio.h>

struct wl_buffer *wp_wl_create_dmabuf_buffer(wp_wl_state_t *state, int fd,
                                             uint32_t width, uint32_t height,
                                             uint32_t stride,
                                             uint64_t modifier) {
  if (!state->dmabuf) {
    printf("zwp_linux_dmabuf_v1 not available\n");
    return NULL;
  }

  struct zwp_linux_buffer_params_v1 *params =
      zwp_linux_dmabuf_v1_create_params(state->dmabuf);
  if (!params) {
    return NULL;
  }

  uint32_t modifier_hi = (uint32_t)(modifier >> 32);
  uint32_t modifier_lo = (uint32_t)(modifier & 0xffffffffu);
  zwp_linux_buffer_params_v1_add(params, fd, 0, 0, stride, modifier_hi,
                                 modifier_lo);

  struct wl_buffer *buffer = zwp_linux_buffer_params_v1_create_immed(
      params, (int32_t)width, (int32_t)height, WP_WL_DRM_FORMAT_XRGB8888, 0);
  zwp_linux_buffer_params_v1_destroy(params);

  return buffer;
}
