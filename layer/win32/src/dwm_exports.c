#include "dwm_exports.h"

typedef BOOL(WINAPI *DwmIsCompositionEnabled_t)(BOOL *);
typedef HRESULT(WINAPI *DwmGetWindowAttribute_t)(HWND, DWORD, PVOID, DWORD);
typedef HRESULT(WINAPI *DwmSetWindowAttribute_t)(HWND, DWORD, LPCVOID, DWORD);
typedef BOOL(WINAPI *DwmDefWindowProc_t)(HWND, UINT, WPARAM, LPARAM, LRESULT *);

static DwmIsCompositionEnabled_t real_DwmIsCompositionEnabled;
static DwmGetWindowAttribute_t real_DwmGetWindowAttribute;
static DwmSetWindowAttribute_t real_DwmSetWindowAttribute;
static DwmDefWindowProc_t real_DwmDefWindowProc;

void load_real_dwmapi(void) {
  WCHAR path[MAX_PATH];
  DWORD len = GetEnvironmentVariableW(L"WALLPIPER_DWMAPI_ORIG", path, MAX_PATH);
  if (len == 0 || len >= MAX_PATH) {
    return;
  }

  HMODULE real = LoadLibraryW(path);
  if (!real) {
    return;
  }

  real_DwmIsCompositionEnabled =
      (DwmIsCompositionEnabled_t)(void *)GetProcAddress(
          real, "DwmIsCompositionEnabled");
  real_DwmGetWindowAttribute = (DwmGetWindowAttribute_t)(void *)GetProcAddress(
      real, "DwmGetWindowAttribute");
  real_DwmSetWindowAttribute = (DwmSetWindowAttribute_t)(void *)GetProcAddress(
      real, "DwmSetWindowAttribute");
  real_DwmDefWindowProc =
      (DwmDefWindowProc_t)(void *)GetProcAddress(real, "DwmDefWindowProc");
}

WP_DLLEXPORT BOOL WINAPI DwmIsCompositionEnabled(BOOL *pfEnabled) {
  if (real_DwmIsCompositionEnabled) {
    return real_DwmIsCompositionEnabled(pfEnabled);
  }
  if (pfEnabled) {
    *pfEnabled = TRUE;
  }
  return S_OK;
}

WP_DLLEXPORT HRESULT WINAPI DwmGetWindowAttribute(HWND hwnd, DWORD attr,
                                                  PVOID pv, DWORD cb) {
  if (real_DwmGetWindowAttribute) {
    return real_DwmGetWindowAttribute(hwnd, attr, pv, cb);
  }
  return E_NOTIMPL;
}

WP_DLLEXPORT HRESULT WINAPI DwmSetWindowAttribute(HWND hwnd, DWORD attr,
                                                  LPCVOID pv, DWORD cb) {
  if (real_DwmSetWindowAttribute) {
    return real_DwmSetWindowAttribute(hwnd, attr, pv, cb);
  }
  return E_NOTIMPL;
}

WP_DLLEXPORT BOOL WINAPI DwmDefWindowProc(HWND hwnd, UINT msg, WPARAM wparam,
                                          LPARAM lparam, LRESULT *result) {
  if (real_DwmDefWindowProc) {
    return real_DwmDefWindowProc(hwnd, msg, wparam, lparam, result);
  }
  return FALSE;
}
