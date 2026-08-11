#pragma once
#include "portal_state.h"

G_BEGIN_DECLS
gboolean wallpiper_ctl_listener_start(WallpiperPortalState* state, GError** error);
void wallpiper_ctl_listener_stop(WallpiperPortalState* state);
G_END_DECLS
