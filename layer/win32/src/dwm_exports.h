#ifndef WP_DWM_EXPORTS_H
#define WP_DWM_EXPORTS_H

#include "win32/windows.h"

void load_real_dwmapi(void);

WP_DLLEXPORT BOOL WINAPI DwmIsCompositionEnabled(BOOL *pfEnabled);
WP_DLLEXPORT HRESULT WINAPI DwmGetWindowAttribute(HWND hwnd, DWORD attr,
                                                  PVOID pv, DWORD cb);
WP_DLLEXPORT HRESULT WINAPI DwmSetWindowAttribute(HWND hwnd, DWORD attr,
                                                  LPCVOID pv, DWORD cb);
WP_DLLEXPORT BOOL WINAPI DwmDefWindowProc(HWND hwnd, UINT msg, WPARAM wparam,
                                          LPARAM lparam, LRESULT *result);

#endif
