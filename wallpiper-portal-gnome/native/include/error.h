#pragma once

#include <glib.h>

G_BEGIN_DECLS
GQuark wallpiper_error_quark(void);
#define WALLPIPER_ERROR wallpiper_error_quark()
G_END_DECLS
