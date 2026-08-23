#include "actor_stacking.h"

ClutterActor *wallpiper_actor_stacking_dump_children(ClutterActor *parent,
                                                     const char *tag) {
  guint n = 0;
  ClutterActor *background_actor = NULL;
  for (ClutterActor *child = clutter_actor_get_first_child(parent);
       child != NULL; child = clutter_actor_get_next_sibling(child), n++) {
    const char *type_name = G_OBJECT_TYPE_NAME(child);
    g_message("wallpiper-gnome: [%s] window_group child[%u] = %s (%p) \"%s\"",
              tag, n, type_name, (void *)child, clutter_actor_get_name(child));
    if (!background_actor && g_strcmp0(type_name, "MetaBackgroundGroup") == 0)
      background_actor = child;
  }
  g_message(
      "wallpiper-gnome: [%s] window_group has %u children, background_actor=%p",
      tag, n, (void *)background_actor);
  return background_actor;
}

void wallpiper_actor_stacking_reassert(WallpiperPortalState *state,
                                       const char *tag) {
  if (!state->parent)
    return;

  ClutterActor *background_actor = NULL;
  for (ClutterActor *child = clutter_actor_get_first_child(state->parent);
       child != NULL; child = clutter_actor_get_next_sibling(child)) {
    if (g_strcmp0(G_OBJECT_TYPE_NAME(child), "MetaBackgroundGroup") == 0) {
      background_actor = child;
      break;
    }
  }

  ClutterActor *reference = background_actor
                                ? background_actor
                                : clutter_actor_get_first_child(state->parent);
  if (!reference)
    return;

  for (int i = 0; i < WP_MAX_CAPTURE_CHANNELS; i++) {
    ClutterActor *actor = state->channels[i].display_actor;
    if (!actor || actor == reference)
      continue;

    if (clutter_actor_get_previous_sibling(actor) == reference)
      continue; /* already correctly positioned */

    clutter_actor_set_child_above_sibling(state->parent, actor, reference);
    g_message("wallpiper-gnome: [%s] reasserted channel %d display_actor "
              "position above background",
              tag, i);
  }
}

static void on_display_restacked(MetaDisplay *display, gpointer user_data) {
  (void)display;
  wallpiper_actor_stacking_reassert((WallpiperPortalState *)user_data,
                                    "restacked");
}

static void on_window_created(MetaDisplay *display, MetaWindow *window,
                              gpointer user_data) {
  (void)display;
  (void)window;
  wallpiper_actor_stacking_reassert((WallpiperPortalState *)user_data,
                                    "window-created");
}

void wallpiper_actor_stacking_connect(WallpiperPortalState *state) {
  if (!state->meta_display) {
    g_warning("wallpiper-gnome: could not get MetaDisplay, actor position "
              "won't be reasserted on restack");
    return;
  }

  state->restacked_handler_id =
      g_signal_connect(state->meta_display, "restacked",
                       G_CALLBACK(on_display_restacked), state);
  state->window_created_handler_id =
      g_signal_connect(state->meta_display, "window-created",
                       G_CALLBACK(on_window_created), state);
}

void wallpiper_actor_stacking_disconnect(WallpiperPortalState *state) {
  if (!state->meta_display)
    return;

  g_signal_handler_disconnect(state->meta_display, state->restacked_handler_id);
  g_signal_handler_disconnect(state->meta_display,
                              state->window_created_handler_id);
}
