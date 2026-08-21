#ifndef WP_PE_IAT_H
#define WP_PE_IAT_H

#include "win32/windows.h"

FARPROC patch_iat(HMODULE image, const char *dll_name, const char *func_name,
                  FARPROC replacement);
int patch_iat_all_modules(const char *dll_name, const char *func_name,
                          FARPROC replacement, FARPROC *out_original);

#endif
