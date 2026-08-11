#pragma once
#include "portal_state.h"

G_BEGIN_DECLS
gboolean wallpiper_capture_listener_start(WallpiperPortalState* state, GError** error);
void wallpiper_capture_listener_stop(WallpiperPortalState* state);
void wallpiper_capture_listener_detach(WallpiperPortalState* state);
G_END_DECLS
