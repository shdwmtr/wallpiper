#ifndef WP_PROGMAN_H
#define WP_PROGMAN_H

#include "win32/windows.h"

extern HWND g_fake_icon_workerw;
extern HWND g_fake_empty_workerw;

void reassert_workerw_zorder(void);
void install_progman_hook(void);

#endif
