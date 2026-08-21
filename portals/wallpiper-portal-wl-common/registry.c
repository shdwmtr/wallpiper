#include "state_internal.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static uint32_t min_u32(uint32_t a, uint32_t b) { return a < b ? a : b; }
#define MIN_U32(a, b) min_u32((uint32_t)(a), (uint32_t)(b))

static void registry_global(void *data, struct wl_registry *registry,
                            uint32_t name, const char *interface,
                            uint32_t version) {
  wp_wl_state_t *state = data;

  if (strcmp(interface, wl_compositor_interface.name) == 0) {
    state->compositor =
        wl_registry_bind(registry, name, &wl_compositor_interface,
                         MIN_U32(version, wl_compositor_interface.version));
  } else if (strcmp(interface, zwlr_layer_shell_v1_interface.name) == 0) {
    state->layer_shell = wl_registry_bind(
        registry, name, &zwlr_layer_shell_v1_interface,
        MIN_U32(version, zwlr_layer_shell_v1_interface.version));
  } else if (strcmp(interface, zwp_linux_dmabuf_v1_interface.name) == 0) {
    if (version >= WP_WL_DMABUF_MIN_VERSION) {
      state->dmabuf = wl_registry_bind(
          registry, name, &zwp_linux_dmabuf_v1_interface,
          MIN_U32(version, zwp_linux_dmabuf_v1_interface.version));
    } else {
      printf("zwp_linux_dmabuf_v1 version %u too old (need >= %d), dmabuf "
             "capture will not display\n",
             version, WP_WL_DMABUF_MIN_VERSION);
    }
  } else if (strcmp(interface, wp_viewporter_interface.name) == 0) {
    state->viewporter =
        wl_registry_bind(registry, name, &wp_viewporter_interface,
                         MIN_U32(version, wp_viewporter_interface.version));
  } else if (strcmp(interface, wl_shm_interface.name) == 0) {
    state->shm = wl_registry_bind(registry, name, &wl_shm_interface,
                                  MIN_U32(version, wl_shm_interface.version));
  } else if (strcmp(interface, wl_subcompositor_interface.name) == 0) {
    state->subcompositor =
        wl_registry_bind(registry, name, &wl_subcompositor_interface,
                         MIN_U32(version, wl_subcompositor_interface.version));
  } else if (strcmp(interface, wl_output_interface.name) == 0) {
    struct wl_output *output =
        wl_registry_bind(registry, name, &wl_output_interface,
                         MIN_U32(version, wl_output_interface.version));
    wp_wl_attach_output_listener(state, output);
  }
}

static void registry_global_remove(void *data, struct wl_registry *registry,
                                   uint32_t name) {
  (void)data;
  (void)registry;
  (void)name;
}

static const struct wl_registry_listener REGISTRY_LISTENER = {
    .global = registry_global,
    .global_remove = registry_global_remove,
};

void wp_wl_bind_globals(wp_wl_state_t *state) {
  state->registry = wl_display_get_registry(state->display);
  wl_registry_add_listener(state->registry, &REGISTRY_LISTENER, state);
  wl_display_roundtrip(state->display);

  if (!state->compositor) {
    fprintf(stderr, "wl_compositor not available\n");
    exit(1);
  }
  if (!state->layer_shell) {
    fprintf(stderr, "zwlr_layer_shell_v1 not available\n");
    exit(1);
  }
  if (!state->dmabuf) {
    printf("zwp_linux_dmabuf_v1 not available, dmabuf-sourced frames won't "
           "display\n");
  }
  if (!state->viewporter) {
    printf("wp_viewporter not available, buffer will be shown at native size "
           "(may overhang on "
           "fractional-scale outputs)\n");
  }
  if (!state->shm) {
    printf("wl_shm not available, SHM-sourced frames (e.g. web wallpapers) "
           "won't display\n");
  }
  if (!state->subcompositor) {
    printf(
        "wl_subcompositor not available, debug overlay won't be available\n");
  }
}
