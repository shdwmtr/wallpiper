#ifndef WP_WIN32_BOOTSTRAP_H
#define WP_WIN32_BOOTSTRAP_H

#include "types.h"

PVOID win32_bootstrap_find_kernel32(void);
FARPROC win32_bootstrap_find_export(PVOID module_base, const char *name);

#endif
