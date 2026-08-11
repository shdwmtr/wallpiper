#include "wallpiper-portal.h"

#include "internal/actor-stacking.h"
#include "internal/capture-listener.h"
#include "internal/ctl-listener.h"
#include "internal/error.h"
#include "internal/monitor-geometry.h"
#include "internal/mutter-private.h"
#include "internal/portal-state.h"
#include "internal/protocol.h"

#include <meta/meta-context.h>

static WallpiperPortalState *portal_state = NULL;

gboolean wallpiper_portal_start(GObject *backend_obj, GObject *parent_obj,
                                GError **error) {
  if (portal_state) {
    g_set_error(error, WALLPIPER_ERROR, 0, "portal already running");
    return FALSE;
  }

  if (!META_IS_BACKEND(backend_obj)) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "backend argument is not a MetaBackend");
    return FALSE;
  }

  if (!CLUTTER_IS_ACTOR(parent_obj)) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "parent argument is not a ClutterActor");
    return FALSE;
  }

  MetaBackend *backend = META_BACKEND(backend_obj);
  ClutterBackend *clutter_backend = meta_backend_get_clutter_backend(backend);
  CoglContext *cogl_context = clutter_backend_get_cogl_context(clutter_backend);
  EGLDisplay egl_display = cogl_context_get_egl_display(cogl_context);

  if (!egl_display) {
    g_set_error(error, WALLPIPER_ERROR, 0,
                "could not get Mutter's EGLDisplay from Cogl");
    return FALSE;
  }

  WallpiperMonitorGeometry geometry = wallpiper_monitor_detect_primary(backend);
  g_message("wallpiper-gnome: detected monitor geometry %ux%u at (%d,%d), "
            "logical %ux%u",
            geometry.width, geometry.height, geometry.x, geometry.y,
            geometry.logical_width, geometry.logical_height);

  WallpiperPortalState *state = g_new0(WallpiperPortalState, 1);
  state->backend = backend;
  state->cogl_context = cogl_context;
  state->egl_display = egl_display;
  state->geometry = geometry;

  ClutterActor *parent = CLUTTER_ACTOR(parent_obj);
  ClutterActor *actor = clutter_actor_new();
  clutter_actor_set_position(actor, geometry.x, geometry.y);
  clutter_actor_set_size(actor, geometry.width, geometry.height);
  clutter_actor_show(actor);

  ClutterActor *background_actor =
      wallpiper_actor_stacking_dump_children(parent, "enable");
  if (background_actor)
    clutter_actor_insert_child_above(parent, actor, background_actor);
  else {
    ClutterActor *current_bottom = clutter_actor_get_first_child(parent);
    if (current_bottom)
      clutter_actor_insert_child_above(parent, actor, current_bottom);
    else
      clutter_actor_add_child(parent, actor);
  }

  state->parent = parent;
  state->display_actor = actor;

  MetaContext *meta_context = meta_backend_get_context(backend);
  state->meta_display =
      meta_context ? meta_context_get_display(meta_context) : NULL;
  wallpiper_actor_stacking_connect(state);

  if (!wallpiper_capture_listener_start(state, error)) {
    wallpiper_actor_stacking_disconnect(state);
    clutter_actor_destroy(state->display_actor);
    g_free(state);
    return FALSE;
  }

  if (!wallpiper_ctl_listener_start(state, error)) {
    wallpiper_capture_listener_stop(state);
    wallpiper_actor_stacking_disconnect(state);
    clutter_actor_destroy(state->display_actor);
    g_free(state);
    return FALSE;
  }

  portal_state = state;

  g_message(
      "wallpiper-gnome: portal started — capture socket %s, ctl socket %s",
      WALLPIPER_CAPTURE_SOCKET_PATH, WALLPIPER_CTL_SOCKET_PATH);

  return TRUE;
}

void wallpiper_portal_stop(void) {
  if (!portal_state)
    return;

  wallpiper_ctl_listener_stop(portal_state);
  wallpiper_capture_listener_stop(portal_state);
  wallpiper_actor_stacking_disconnect(portal_state);

  if (portal_state->display_actor)
    clutter_actor_destroy(portal_state->display_actor);

  g_free(portal_state);
  portal_state = NULL;

  g_message("wallpiper-gnome: portal stopped");
}
