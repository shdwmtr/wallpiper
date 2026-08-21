#include "state_internal.h"

#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

static void handle_buf(wp_wl_state_t *state, uint32_t slot, uint32_t width,
                       uint32_t height, uint32_t stride, uint64_t modifier,
                       int fd) {
  for (size_t i = 0; i < WP_WL_MAX_SLOTS; i++) {
    if (state->slots[i].in_use && state->slots[i].slot == slot) {
      wl_buffer_destroy(state->slots[i].buffer);
      close(state->slots[i].fd);
      state->slots[i].in_use = false;
      break;
    }
  }

  struct wl_buffer *buffer =
      wp_wl_create_dmabuf_buffer(state, fd, width, height, stride, modifier);
  if (!buffer) {
    close(fd);
    return;
  }

  int free_index = -1;
  for (size_t i = 0; i < WP_WL_MAX_SLOTS; i++) {
    if (!state->slots[i].in_use) {
      free_index = (int)i;
      break;
    }
  }
  if (free_index < 0) {
    printf("[socket] slot table full, dropping capture slot %u\n", slot);
    wl_buffer_destroy(buffer);
    close(fd);
    return;
  }

  state->slots[free_index].in_use = true;
  state->slots[free_index].slot = slot;
  state->slots[free_index].buffer = buffer;
  state->slots[free_index].fd = fd;
  state->slots[free_index].width = width;
  state->slots[free_index].height = height;

  printf("[socket] registered capture slot %u %ux%u stride=%u modifier=%llu\n",
         slot, width, height, stride, (unsigned long long)modifier);

  wp_wl_source_t source = {.kind = WP_WL_SOURCE_SLOT, .slot = slot};
  wp_wl_set_current_source(state, source);
}

void wp_wl_set_current_source(wp_wl_state_t *state, wp_wl_source_t source) {
  state->current_source = source;
  state->has_current_source = true;
  if (!state->frame_loop_running) {
    state->frame_loop_running = true;
    wp_wl_refresh_buffer(state);
  }
}

void wp_wl_handle_capture_event(wp_wl_state_t *state,
                                const wp_capture_event_t *event) {
  wp_frame_stats_record_capture(state->stats);

  switch (event->tag) {
  case WP_CAPTURE_EVENT_BUF: {
    int image_fd = event->fds[0];
    if (event->nfds > 1) {
      close(event->fds[1]);
    }
    handle_buf(state, event->slot, event->width, event->height, event->stride,
               event->modifier, image_fd);
    break;
  }
  case WP_CAPTURE_EVENT_FRAME: {
    if (event->nfds > 0) {
      close(event->fds[0]);
    }
    for (size_t i = 0; i < WP_WL_MAX_SLOTS; i++) {
      if (state->slots[i].in_use && state->slots[i].slot == event->slot) {
        wp_wl_source_t source = {.kind = WP_WL_SOURCE_SLOT,
                                 .slot = event->slot};
        wp_wl_set_current_source(state, source);
        break;
      }
    }
    break;
  }
  case WP_CAPTURE_EVENT_SHM: {
    wp_wl_source_t source = {
        .kind = WP_WL_SOURCE_SHM,
        .fd = event->fds[0],
        .width = event->width,
        .height = event->height,
        .stride = event->stride,
    };
    wp_wl_set_current_source(state, source);
    break;
  }
  }
}

void wp_wl_detach(wp_wl_state_t *state) {
  if (state->surface) {
    wl_surface_attach(state->surface, NULL, 0, 0);
    wl_surface_commit(state->surface);
  }
  state->has_current_source = false;
  state->frame_loop_running = false;

  if (state->current_shm_buffer) {
    wl_buffer_destroy(state->current_shm_buffer);
    state->current_shm_buffer = NULL;
  }
  if (state->current_shm_pool) {
    wl_shm_pool_destroy(state->current_shm_pool);
    state->current_shm_pool = NULL;
  }

  for (size_t i = 0; i < WP_WL_MAX_SLOTS; i++) {
    if (state->slots[i].in_use) {
      wl_buffer_destroy(state->slots[i].buffer);
      close(state->slots[i].fd);
      state->slots[i].in_use = false;
    }
  }

  printf("[ctl] detached from compositor, released all buffers\n");
}

void wp_wl_refresh_buffer(wp_wl_state_t *state) {
  if (!state->surface || !state->has_current_source) {
    return;
  }

  struct wl_buffer *buffer = NULL;
  uint32_t width = 0;
  uint32_t height = 0;
  struct wl_buffer *next_shm_buffer = NULL;

  if (state->current_source.kind == WP_WL_SOURCE_SLOT) {
    wp_wl_slot_t *found = NULL;
    for (size_t i = 0; i < WP_WL_MAX_SLOTS; i++) {
      if (state->slots[i].in_use &&
          state->slots[i].slot == state->current_source.slot) {
        found = &state->slots[i];
        break;
      }
    }
    if (!found) {
      return;
    }
    buffer = found->buffer;
    width = found->width;
    height = found->height;
  } else if (state->current_source.kind == WP_WL_SOURCE_SHM) {
    if (!state->shm) {
      printf("wl_shm not available\n");
      return;
    }
    int32_t pool_size = (int32_t)state->current_source.stride *
                        (int32_t)state->current_source.height;
    struct wl_shm_pool *pool =
        wl_shm_create_pool(state->shm, state->current_source.fd, pool_size);
    struct wl_buffer *shm_buffer = wl_shm_pool_create_buffer(
        pool, 0, (int32_t)state->current_source.width,
        (int32_t)state->current_source.height,
        (int32_t)state->current_source.stride, WL_SHM_FORMAT_XRGB8888);

    if (state->current_shm_pool) {
      wl_shm_pool_destroy(state->current_shm_pool);
    }
    state->current_shm_pool = pool;

    buffer = shm_buffer;
    width = state->current_source.width;
    height = state->current_source.height;
    next_shm_buffer = shm_buffer;
  } else {
    return;
  }

  wl_surface_attach(state->surface, buffer, 0, 0);
  wl_surface_damage_buffer(state->surface, 0, 0, (int32_t)width,
                           (int32_t)height);
  if (state->viewport) {
    wp_viewport_set_destination(state->viewport, (int32_t)state->width,
                                (int32_t)state->height);
  }
  wp_wl_request_frame_callback(state);
  wl_surface_commit(state->surface);

  if (state->current_shm_buffer) {
    wl_buffer_destroy(state->current_shm_buffer);
  }
  state->current_shm_buffer = next_shm_buffer;

  wp_frame_stats_record_display(state->stats);
}

void wp_wl_set_debug_enabled(wp_wl_state_t *state, bool enabled) {
  state->debug_enabled = enabled;
  if (enabled) {
    wp_wl_ensure_debug_surface(state);
    wp_debug_throttle_reset(&state->debug_throttle);
  } else if (state->debug_surface) {
    wl_surface_attach(state->debug_surface, NULL, 0, 0);
    wl_surface_commit(state->debug_surface);
  }
  printf("[ctl] debug overlay -> %s\n", enabled ? "true" : "false");
}

void wp_wl_ensure_debug_surface(wp_wl_state_t *state) {
  if (state->debug_surface) {
    return;
  }
  if (!state->subcompositor) {
    printf("[debug] wl_subcompositor not available, cannot create overlay\n");
    return;
  }
  if (!state->surface) {
    return;
  }

  struct wl_surface *surface = wl_compositor_create_surface(state->compositor);
  struct wl_subsurface *subsurface = wl_subcompositor_get_subsurface(
      state->subcompositor, surface, state->surface);
  int32_t y = ((int32_t)state->height - WP_DEBUG_OVERLAY_HEIGHT) / 2;
  if (y < 0) {
    y = 0;
  }
  wl_subsurface_set_position(subsurface, 12, y);
  wl_subsurface_set_desync(subsurface);
  wl_surface_commit(surface);

  state->debug_surface = surface;
  state->debug_subsurface = subsurface;
}

void wp_wl_maybe_redraw_debug(wp_wl_state_t *state) {
  if (wp_debug_throttle_should_redraw(&state->debug_throttle)) {
    wp_wl_draw_debug_overlay(state);
  }
}

void wp_wl_draw_debug_overlay(wp_wl_state_t *state) {
  wp_wl_ensure_debug_surface(state);
  if (!state->debug_surface || !state->shm) {
    return;
  }

  static uint8_t pixels[WP_DEBUG_OVERLAY_BUFFER_SIZE];
  wp_render_stats_panel(state->stats, pixels);

  int memfd = memfd_create("wallpiper-debug-overlay", 0);
  if (memfd < 0) {
    printf("[debug] memfd_create failed\n");
    return;
  }

  ssize_t written = write(memfd, pixels, sizeof(pixels));
  if (written < 0 || (size_t)written != sizeof(pixels)) {
    printf("[debug] failed to write overlay pixels\n");
    close(memfd);
    return;
  }

  struct wl_shm_pool *pool =
      wl_shm_create_pool(state->shm, memfd, WP_DEBUG_OVERLAY_BUFFER_SIZE);
  struct wl_buffer *buffer = wl_shm_pool_create_buffer(
      pool, 0, WP_DEBUG_OVERLAY_WIDTH, WP_DEBUG_OVERLAY_HEIGHT,
      WP_DEBUG_OVERLAY_STRIDE, WL_SHM_FORMAT_ARGB8888);

  wl_surface_attach(state->debug_surface, buffer, 0, 0);
  wl_surface_damage_buffer(state->debug_surface, 0, 0, WP_DEBUG_OVERLAY_WIDTH,
                           WP_DEBUG_OVERLAY_HEIGHT);
  wl_surface_commit(state->debug_surface);

  if (state->debug_shm_pool) {
    wl_shm_pool_destroy(state->debug_shm_pool);
  }
  state->debug_shm_pool = pool;

  if (state->debug_shm_buffer) {
    wl_buffer_destroy(state->debug_shm_buffer);
  }
  state->debug_shm_buffer = buffer;

  /* The compositor won't actually see this request (and mmap the fd) until the
   * next wl_display_flush() in the main loop, so we can't close memfd here -
   * only the *previous* redraw's fd, which has had a full cycle to be flushed
   * and processed by now. */
  if (state->has_debug_shm_fd) {
    close(state->debug_shm_fd);
  }
  state->debug_shm_fd = memfd;
  state->has_debug_shm_fd = true;
}

void wp_wl_refresh_geometry(wp_wl_state_t *state) {
  wp_monitor_geometry_t geometry;
  wp_wl_detect_geometry(state->config->try_geometry, &geometry);

  if (memcmp(&geometry, &state->geometry, sizeof(geometry)) != 0) {
    printf(
        "monitor geometry changed: x=%d y=%d %ux%u logical=%ux%u scale=%.4f\n",
        geometry.x, geometry.y, geometry.width, geometry.height,
        geometry.logical_width, geometry.logical_height, geometry.scale);
  }
  state->geometry = geometry;
}
