#ifndef WP_TRAY_H
#define WP_TRAY_H

#include "win32/windows.h"

void install_tray_hooks(void);
void post_tray_click(BOOL is_right_click);

extern UINT32 g_tray_uid;
extern UINT32 g_tray_callback_msg;
extern volatile LONG g_tray_icon_version_4;

#endif
