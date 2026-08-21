#include <poll.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#include <xcb/dri3.h>
#include <xcb/shm.h>
#include <xcb/xcb.h>
#include <xcb/xproto.h>

#include <cJSON.h>

#include "wallpiper/capture_socket.h"
#include "wallpiper/ctl_protocol.h"
#include "wallpiper/debug_overlay.h"
#include "wallpiper/monitor_geometry.h"

#define WP_I3_MAX_SLOTS 8

typedef struct {
  bool in_use;
  uint32_t slot;
  xcb_pixmap_t pixmap;
  uint32_t width;
  uint32_t height;
} wp_i3_slot_t;

typedef struct {
  bool valid;
  xcb_pixmap_t pixmap;
  xcb_shm_seg_t seg;
  uint32_t width;
  uint32_t height;
} wp_i3_shm_pixmap_t;

typedef enum {
  WP_I3_SOURCE_NONE,
  WP_I3_SOURCE_SLOT,
  WP_I3_SOURCE_SHM,
} wp_i3_source_kind_t;

typedef struct {
  xcb_connection_t *conn;
  xcb_window_t window;
  xcb_gcontext_t gc;
  uint8_t depth;
  wp_monitor_geometry_t geometry;

  wp_i3_slot_t slots[WP_I3_MAX_SLOTS];
  wp_i3_shm_pixmap_t current_shm_pixmap;
  wp_i3_source_kind_t current_source_kind;
  uint32_t current_source_slot;

  bool debug_enabled;
  wp_debug_throttle_t debug_throttle;
  wp_frame_stats_t *stats;
} wp_i3_state_t;

static char *run_command(const char *const argv[]) {
  int pipefd[2];
  if (pipe(pipefd) != 0) {
    return NULL;
  }

  pid_t pid = fork();
  if (pid < 0) {
    close(pipefd[0]);
    close(pipefd[1]);
    return NULL;
  }

  if (pid == 0) {
    close(pipefd[0]);
    dup2(pipefd[1], STDOUT_FILENO);
    close(pipefd[1]);
    execvp(argv[0], (char *const *)argv);
    _exit(127);
  }

  close(pipefd[1]);

  size_t cap = 65536;
  size_t len = 0;
  char *buf = malloc(cap);
  if (!buf) {
    close(pipefd[0]);
    waitpid(pid, NULL, 0);
    return NULL;
  }

  for (;;) {
    if (len + 4096 > cap) {
      cap *= 2;
      char *grown = realloc(buf, cap);
      if (!grown) {
        free(buf);
        close(pipefd[0]);
        waitpid(pid, NULL, 0);
        return NULL;
      }
      buf = grown;
    }
    ssize_t n = read(pipefd[0], buf + len, 4096);
    if (n <= 0) {
      break;
    }
    len += (size_t)n;
  }
  buf[len] = '\0';
  close(pipefd[0]);

  int status = 0;
  waitpid(pid, &status, 0);
  if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
    free(buf);
    return NULL;
  }

  return buf;
}

static bool try_detect_geometry(wp_monitor_geometry_t *out) {
  const char *ws_argv[] = {"i3-msg", "-t", "get_workspaces", NULL};
  char *ws_output = run_command(ws_argv);
  if (!ws_output) {
    return false;
  }

  cJSON *workspaces = cJSON_Parse(ws_output);
  free(ws_output);
  if (!workspaces || !cJSON_IsArray(workspaces)) {
    cJSON_Delete(workspaces);
    return false;
  }

  const char *output_name = NULL;
  int ws_count = cJSON_GetArraySize(workspaces);
  for (int i = 0; i < ws_count; i++) {
    cJSON *ws = cJSON_GetArrayItem(workspaces, i);
    cJSON *focused = cJSON_GetObjectItem(ws, "focused");
    if (focused && cJSON_IsTrue(focused)) {
      cJSON *output = cJSON_GetObjectItem(ws, "output");
      if (cJSON_IsString(output)) {
        output_name = output->valuestring;
      }
      break;
    }
  }
  if (!output_name) {
    cJSON_Delete(workspaces);
    return false;
  }

  const char *out_argv[] = {"i3-msg", "-t", "get_outputs", NULL};
  char *out_output = run_command(out_argv);
  if (!out_output) {
    cJSON_Delete(workspaces);
    return false;
  }

  cJSON *outputs = cJSON_Parse(out_output);
  free(out_output);
  if (!outputs || !cJSON_IsArray(outputs)) {
    cJSON_Delete(outputs);
    cJSON_Delete(workspaces);
    return false;
  }

  bool found = false;
  int out_count = cJSON_GetArraySize(outputs);
  for (int i = 0; i < out_count; i++) {
    cJSON *output = cJSON_GetArrayItem(outputs, i);
    cJSON *name = cJSON_GetObjectItem(output, "name");
    cJSON *active = cJSON_GetObjectItem(output, "active");
    if (!cJSON_IsString(name) || strcmp(name->valuestring, output_name) != 0) {
      continue;
    }
    if (!active || !cJSON_IsTrue(active)) {
      continue;
    }

    cJSON *rect = cJSON_GetObjectItem(output, "rect");
    cJSON *x = cJSON_GetObjectItem(rect, "x");
    cJSON *y = cJSON_GetObjectItem(rect, "y");
    cJSON *width = cJSON_GetObjectItem(rect, "width");
    cJSON *height = cJSON_GetObjectItem(rect, "height");
    if (!cJSON_IsNumber(x) || !cJSON_IsNumber(y) || !cJSON_IsNumber(width) ||
        !cJSON_IsNumber(height)) {
      break;
    }

    out->x = (int32_t)x->valuedouble;
    out->y = (int32_t)y->valuedouble;
    out->width = (uint32_t)width->valuedouble;
    out->height = (uint32_t)height->valuedouble;
    out->logical_width = out->width;
    out->logical_height = out->height;
    out->scale = 1.0;
    found = true;
    break;
  }

  cJSON_Delete(outputs);
  cJSON_Delete(workspaces);
  return found;
}

static bool detect_geometry(wp_monitor_geometry_t *out) {
  for (int attempt = 1; attempt <= 3; attempt++) {
    if (try_detect_geometry(out)) {
      return true;
    }
    printf("monitor detection attempt %d failed, retrying\n", attempt);
    usleep(500 * 1000);
  }
  printf("monitor detection failed after retries, falling back to 1920x1080 at "
         "0,0\n");
  out->x = 0;
  out->y = 0;
  out->width = 1920;
  out->height = 1080;
  out->logical_width = 1920;
  out->logical_height = 1080;
  out->scale = 1.0;
  return true;
}

static void cursor_pos_fn(void *ctx, wp_ctl_response_t *out) {
  (void)ctx;
  memset(out, 0, sizeof(*out));

  xcb_connection_t *conn = xcb_connect(NULL, NULL);
  if (!conn || xcb_connection_has_error(conn)) {
    if (conn) {
      xcb_disconnect(conn);
    }
    out->tag = WP_CTL_RESPONSE_ERR;
    snprintf(out->err, sizeof(out->err), "%s", "cursor position unavailable");
    return;
  }

  const xcb_setup_t *setup = xcb_get_setup(conn);
  xcb_screen_t *screen = xcb_setup_roots_iterator(setup).data;

  xcb_query_pointer_cookie_t cookie = xcb_query_pointer(conn, screen->root);
  xcb_generic_error_t *err = NULL;
  xcb_query_pointer_reply_t *reply =
      xcb_query_pointer_reply(conn, cookie, &err);

  if (reply) {
    out->tag = WP_CTL_RESPONSE_CURSOR_POS;
    out->cursor_x = reply->root_x;
    out->cursor_y = reply->root_y;
    free(reply);
  } else {
    out->tag = WP_CTL_RESPONSE_ERR;
    snprintf(out->err, sizeof(out->err), "%s", "cursor position unavailable");
  }
  free(err);
  xcb_disconnect(conn);
}

static void refresh_buffer(wp_i3_state_t *state) {
  xcb_pixmap_t pixmap = XCB_NONE;
  uint32_t width = 0;
  uint32_t height = 0;

  if (state->current_source_kind == WP_I3_SOURCE_SLOT) {
    for (size_t i = 0; i < WP_I3_MAX_SLOTS; i++) {
      if (state->slots[i].in_use &&
          state->slots[i].slot == state->current_source_slot) {
        pixmap = state->slots[i].pixmap;
        width = state->slots[i].width;
        height = state->slots[i].height;
        break;
      }
    }
    if (pixmap == XCB_NONE) {
      return;
    }
  } else if (state->current_source_kind == WP_I3_SOURCE_SHM) {
    if (!state->current_shm_pixmap.valid) {
      return;
    }
    pixmap = state->current_shm_pixmap.pixmap;
    width = state->current_shm_pixmap.width;
    height = state->current_shm_pixmap.height;
  } else {
    return;
  }

  xcb_copy_area(state->conn, pixmap, state->window, state->gc, 0, 0, 0, 0,
                (uint16_t)width, (uint16_t)height);
  xcb_flush(state->conn);

  wp_frame_stats_record_display(state->stats);

  if (state->debug_enabled) {
    static uint8_t pixels[WP_DEBUG_OVERLAY_BUFFER_SIZE];
    wp_render_stats_panel(state->stats, pixels);
    int32_t overlay_y =
        ((int32_t)state->geometry.height - WP_DEBUG_OVERLAY_HEIGHT) / 2;
    if (overlay_y < 0) {
      overlay_y = 0;
    }
    xcb_put_image(state->conn, XCB_IMAGE_FORMAT_Z_PIXMAP, state->window,
                  state->gc, WP_DEBUG_OVERLAY_WIDTH, WP_DEBUG_OVERLAY_HEIGHT,
                  12, (int16_t)overlay_y, 0, state->depth, sizeof(pixels),
                  pixels);
    xcb_flush(state->conn);
  }
}

static void set_current_source(wp_i3_state_t *state, wp_i3_source_kind_t kind,
                               uint32_t slot) {
  state->current_source_kind = kind;
  state->current_source_slot = slot;
  refresh_buffer(state);
}

static void draw_debug_overlay(wp_i3_state_t *state) {
  static uint8_t pixels[WP_DEBUG_OVERLAY_BUFFER_SIZE];
  wp_render_stats_panel(state->stats, pixels);
  int32_t overlay_y =
      ((int32_t)state->geometry.height - WP_DEBUG_OVERLAY_HEIGHT) / 2;
  if (overlay_y < 0) {
    overlay_y = 0;
  }
  xcb_put_image(state->conn, XCB_IMAGE_FORMAT_Z_PIXMAP, state->window,
                state->gc, WP_DEBUG_OVERLAY_WIDTH, WP_DEBUG_OVERLAY_HEIGHT, 12,
                (int16_t)overlay_y, 0, state->depth, sizeof(pixels), pixels);
  xcb_flush(state->conn);
}

static void set_debug_enabled(wp_i3_state_t *state, bool enabled) {
  state->debug_enabled = enabled;
  wp_debug_throttle_reset(&state->debug_throttle);
  if (enabled) {
    draw_debug_overlay(state);
  } else {
    refresh_buffer(state);
  }
  printf("[ctl] debug overlay -> %s\n", enabled ? "true" : "false");
}

static void maybe_redraw_debug(wp_i3_state_t *state) {
  if (wp_debug_throttle_should_redraw(&state->debug_throttle)) {
    draw_debug_overlay(state);
  }
}

static void handle_buf(wp_i3_state_t *state, uint32_t slot, uint32_t width,
                       uint32_t height, uint32_t stride, uint64_t modifier,
                       int fd) {
  for (size_t i = 0; i < WP_I3_MAX_SLOTS; i++) {
    if (state->slots[i].in_use && state->slots[i].slot == slot) {
      xcb_free_pixmap(state->conn, state->slots[i].pixmap);
      state->slots[i].in_use = false;
      break;
    }
  }

  xcb_pixmap_t pixmap = xcb_generate_id(state->conn);
  xcb_void_cookie_t cookie = xcb_dri3_pixmap_from_buffers_checked(
      state->conn, pixmap, state->window, 1, (uint16_t)width, (uint16_t)height,
      stride, 0, 0, 0, 0, 0, 0, 0, state->depth, 32, modifier, &fd);
  xcb_generic_error_t *err = xcb_request_check(state->conn, cookie);
  if (err) {
    printf("[socket] dri3 pixmap import failed for slot %u\n", slot);
    free(err);
    return;
  }

  int free_index = -1;
  for (size_t i = 0; i < WP_I3_MAX_SLOTS; i++) {
    if (!state->slots[i].in_use) {
      free_index = (int)i;
      break;
    }
  }
  if (free_index < 0) {
    printf("[socket] slot table full, dropping capture slot %u\n", slot);
    xcb_free_pixmap(state->conn, pixmap);
    return;
  }

  state->slots[free_index].in_use = true;
  state->slots[free_index].slot = slot;
  state->slots[free_index].pixmap = pixmap;
  state->slots[free_index].width = width;
  state->slots[free_index].height = height;

  printf("[socket] registered capture slot %u %ux%u stride=%u modifier=%llu\n",
         slot, width, height, stride, (unsigned long long)modifier);
  set_current_source(state, WP_I3_SOURCE_SLOT, slot);
}

static void handle_shm(wp_i3_state_t *state, uint32_t width, uint32_t height,
                       uint32_t stride, int fd) {
  if (stride != width * 4) {
    printf("[socket] shm frame stride %u doesn't match tightly-packed %ux4, "
           "X11's MIT-SHM CreatePixmap can't "
           "represent padded rows - dropping frame\n",
           stride, width);
    close(fd);
    return;
  }

  xcb_shm_seg_t seg = xcb_generate_id(state->conn);
  xcb_void_cookie_t attach_cookie =
      xcb_shm_attach_fd_checked(state->conn, seg, fd, 1);
  xcb_generic_error_t *attach_err =
      xcb_request_check(state->conn, attach_cookie);
  if (attach_err) {
    printf("[socket] shm attach_fd failed\n");
    free(attach_err);
    return;
  }

  xcb_pixmap_t pixmap = xcb_generate_id(state->conn);
  xcb_void_cookie_t cookie = xcb_shm_create_pixmap_checked(
      state->conn, pixmap, state->window, (uint16_t)width, (uint16_t)height,
      state->depth, seg, 0);
  xcb_generic_error_t *err = xcb_request_check(state->conn, cookie);
  if (err) {
    printf("[socket] shm pixmap import failed\n");
    free(err);
    xcb_shm_detach(state->conn, seg);
    return;
  }

  if (state->current_shm_pixmap.valid) {
    xcb_free_pixmap(state->conn, state->current_shm_pixmap.pixmap);
    xcb_shm_detach(state->conn, state->current_shm_pixmap.seg);
  }
  state->current_shm_pixmap.valid = true;
  state->current_shm_pixmap.pixmap = pixmap;
  state->current_shm_pixmap.seg = seg;
  state->current_shm_pixmap.width = width;
  state->current_shm_pixmap.height = height;

  set_current_source(state, WP_I3_SOURCE_SHM, 0);
}

static void handle_capture_event(wp_i3_state_t *state,
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
    for (size_t i = 0; i < WP_I3_MAX_SLOTS; i++) {
      if (state->slots[i].in_use && state->slots[i].slot == event->slot) {
        set_current_source(state, WP_I3_SOURCE_SLOT, event->slot);
        break;
      }
    }
    break;
  }
  case WP_CAPTURE_EVENT_SHM: {
    handle_shm(state, event->width, event->height, event->stride,
               event->fds[0]);
    break;
  }
  }
}

static void detach(wp_i3_state_t *state) {
  for (size_t i = 0; i < WP_I3_MAX_SLOTS; i++) {
    if (state->slots[i].in_use) {
      xcb_free_pixmap(state->conn, state->slots[i].pixmap);
      state->slots[i].in_use = false;
    }
  }
  if (state->current_shm_pixmap.valid) {
    xcb_free_pixmap(state->conn, state->current_shm_pixmap.pixmap);
    xcb_shm_detach(state->conn, state->current_shm_pixmap.seg);
    state->current_shm_pixmap.valid = false;
  }
  state->current_source_kind = WP_I3_SOURCE_NONE;

  uint32_t value_list[1] = {0};
  xcb_change_window_attributes(state->conn, state->window, XCB_CW_BACK_PIXEL,
                               value_list);
  xcb_clear_area(state->conn, 0, state->window, 0, 0, 0, 0);
  xcb_flush(state->conn);
  printf("[ctl] detached, released all buffers\n");
}

static void handle_x_event(wp_i3_state_t *state, xcb_generic_event_t *event) {
  uint8_t response_type = event->response_type & 0x7f;
  if (response_type == XCB_EXPOSE) {
    xcb_expose_event_t *expose = (xcb_expose_event_t *)event;
    if (expose->window == state->window && expose->count == 0) {
      refresh_buffer(state);
    }
  } else if (response_type == XCB_DESTROY_NOTIFY) {
    xcb_destroy_notify_event_t *destroy = (xcb_destroy_notify_event_t *)event;
    if (destroy->window == state->window) {
      printf("background window destroyed, exiting\n");
      exit(0);
    }
  } else if (response_type == 0) {
    printf("[x11] protocol error\n");
  }
}

static void handle_ctl_request(wp_i3_state_t *state, wp_ctl_request_t request,
                               wp_ctl_listener_t *listener) {
  wp_ctl_response_t response;
  memset(&response, 0, sizeof(response));

  switch (request) {
  case WP_CTL_REQUEST_GEOMETRY:
    response.tag = WP_CTL_RESPONSE_GEOMETRY;
    response.geometry = state->geometry;
    break;
  case WP_CTL_REQUEST_DETACH:
    detach(state);
    response.tag = WP_CTL_RESPONSE_OK;
    break;
  case WP_CTL_REQUEST_DEBUG_ON:
    set_debug_enabled(state, true);
    response.tag = WP_CTL_RESPONSE_OK;
    break;
  case WP_CTL_REQUEST_DEBUG_OFF:
    set_debug_enabled(state, false);
    response.tag = WP_CTL_RESPONSE_OK;
    break;
  case WP_CTL_REQUEST_CURSOR_POS:
    response.tag = WP_CTL_RESPONSE_ERR;
    snprintf(response.err, sizeof(response.err), "%s",
             "handled by ctl listener");
    break;
  default:
    response.tag = WP_CTL_RESPONSE_ERR;
    snprintf(response.err, sizeof(response.err), "%s", "unrecognized command");
    break;
  }

  wp_ctl_listener_reply(listener, &response);
}

int main(void) {
  setvbuf(stdout, NULL, _IOLBF, 0);

  wp_i3_state_t state;
  memset(&state, 0, sizeof(state));
  state.stats = wp_frame_stats_create();
  wp_debug_throttle_init(&state.debug_throttle);

  detect_geometry(&state.geometry);
  printf(
      "detected monitor geometry: x=%d y=%d %ux%u logical=%ux%u scale=%.4f\n",
      state.geometry.x, state.geometry.y, state.geometry.width,
      state.geometry.height, state.geometry.logical_width,
      state.geometry.logical_height, state.geometry.scale);

  int screen_num = 0;
  state.conn = xcb_connect(NULL, &screen_num);
  if (!state.conn || xcb_connection_has_error(state.conn)) {
    fprintf(stderr, "connect to X11: failed\n");
    return 1;
  }

  const xcb_setup_t *setup = xcb_get_setup(state.conn);
  xcb_screen_iterator_t screen_iter = xcb_setup_roots_iterator(setup);
  for (int i = 0; i < screen_num; i++) {
    xcb_screen_next(&screen_iter);
  }
  xcb_screen_t *screen = screen_iter.data;
  xcb_window_t root = screen->root;
  state.depth = screen->root_depth;
  xcb_visualid_t visual = screen->root_visual;

  xcb_generic_error_t *err = NULL;

  xcb_dri3_query_version_cookie_t dri3_cookie =
      xcb_dri3_query_version(state.conn, 1, 2);
  xcb_dri3_query_version_reply_t *dri3_reply =
      xcb_dri3_query_version_reply(state.conn, dri3_cookie, &err);
  if (!dri3_reply) {
    fprintf(stderr, "DRI3 extension not available\n");
    return 1;
  }
  free(dri3_reply);

  xcb_shm_query_version_cookie_t shm_cookie = xcb_shm_query_version(state.conn);
  xcb_shm_query_version_reply_t *shm_reply =
      xcb_shm_query_version_reply(state.conn, shm_cookie, &err);
  if (!shm_reply) {
    fprintf(stderr, "MIT-SHM extension not available\n");
    return 1;
  }
  free(shm_reply);

  state.window = xcb_generate_id(state.conn);
  uint32_t win_value_list[4] = {0, XCB_BACKING_STORE_ALWAYS, 1,
                                XCB_EVENT_MASK_EXPOSURE};
  uint32_t win_value_mask = XCB_CW_BACK_PIXEL | XCB_CW_BACKING_STORE |
                            XCB_CW_OVERRIDE_REDIRECT | XCB_CW_EVENT_MASK;
  xcb_void_cookie_t create_cookie = xcb_create_window_checked(
      state.conn, state.depth, state.window, root, (int16_t)state.geometry.x,
      (int16_t)state.geometry.y, (uint16_t)state.geometry.width,
      (uint16_t)state.geometry.height, 0, XCB_WINDOW_CLASS_INPUT_OUTPUT, visual,
      win_value_mask, win_value_list);
  xcb_generic_error_t *create_err =
      xcb_request_check(state.conn, create_cookie);
  if (create_err) {
    fprintf(stderr, "create_window failed\n");
    free(create_err);
    return 1;
  }

  xcb_map_window(state.conn, state.window);

  uint32_t stack_value_list[1] = {XCB_STACK_MODE_BELOW};
  xcb_configure_window(state.conn, state.window, XCB_CONFIG_WINDOW_STACK_MODE,
                       stack_value_list);

  state.gc = xcb_generate_id(state.conn);
  xcb_create_gc(state.conn, state.gc, state.window, 0, NULL);
  xcb_flush(state.conn);

  printf("background window %u mapped at %ux%u+%d+%d\n", state.window,
         state.geometry.width, state.geometry.height, state.geometry.x,
         state.geometry.y);

  int capture_fd = wp_bind_capture_socket();
  wp_ctl_listener_t *ctl_listener =
      wp_ctl_listener_start("i3", cursor_pos_fn, NULL);

  for (;;) {
    xcb_generic_event_t *event;
    while ((event = xcb_poll_for_event(state.conn)) != NULL) {
      handle_x_event(&state, event);
      free(event);
    }

    for (;;) {
      wp_capture_event_t capture_event;
      if (!wp_recv_capture_event(capture_fd, &capture_event)) {
        break;
      }
      handle_capture_event(&state, &capture_event);
    }

    wp_ctl_request_t request;
    if (ctl_listener && wp_ctl_listener_poll(ctl_listener, &request)) {
      handle_ctl_request(&state, request, ctl_listener);
    }

    if (state.debug_enabled) {
      maybe_redraw_debug(&state);
    }

    xcb_flush(state.conn);

    struct pollfd pollfds[2] = {
        {.fd = capture_fd, .events = POLLIN, .revents = 0},
        {.fd = xcb_get_file_descriptor(state.conn),
         .events = POLLIN,
         .revents = 0},
    };
    poll(pollfds, 2, 250);
  }
}
