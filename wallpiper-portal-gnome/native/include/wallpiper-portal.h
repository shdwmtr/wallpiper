#pragma once

#include <glib-object.h>

#define HAVE_EGL 1
#include <clutter/clutter.h>

G_BEGIN_DECLS
gboolean wallpiper_place_test_actor(GObject *backend, GObject *parent,
                                    GObject *below_sibling, int x, int y,
                                    int width, int height, GError **error);
gboolean wallpiper_portal_start(GObject *backend, GObject *parent,
                                GError **error);
void wallpiper_portal_stop(void);
G_END_DECLS
