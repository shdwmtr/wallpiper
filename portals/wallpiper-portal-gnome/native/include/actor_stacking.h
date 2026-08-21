#pragma once

#include "portal_state.h"

G_BEGIN_DECLS
ClutterActor *wallpiper_actor_stacking_dump_children(ClutterActor *parent,
                                                     const char *tag);

void wallpiper_actor_stacking_reassert(WallpiperPortalState *state,
                                       const char *tag);
void wallpiper_actor_stacking_connect(WallpiperPortalState *state);
void wallpiper_actor_stacking_disconnect(WallpiperPortalState *state);
G_END_DECLS
