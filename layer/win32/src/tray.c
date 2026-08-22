#include "tray.h"
#include "cursor.h"
#include "pe_iat.h"
#include "util.h"
#include <stdlib.h>

UINT32 g_tray_uid;
UINT32 g_tray_callback_msg;
volatile LONG g_tray_icon_version_4;

static BOOL get_tray_icon_file_path(WCHAR *out, DWORD out_len) {
  return GetEnvironmentVariableW(L"WALLPIPER_TRAY_ICON_FILE", out, out_len) > 0;
}

static void write_tray_icon_file(PNOTIFYICONDATAW nid) {
  WCHAR path[MAX_PATH];
  if (!get_tray_icon_file_path(path, MAX_PATH)) {
    return;
  }
  if (!(nid->uFlags & NIF_ICON) || !nid->hIcon) {
    return;
  }

  ICONINFO info;
  if (!GetIconInfo(nid->hIcon, &info)) {
    debug_log("write_tray_icon_file: GetIconInfo failed");
    return;
  }

  BITMAP bmp;
  if (!GetObjectW((HGDIOBJ)info.hbmColor, sizeof(bmp), &bmp)) {
    debug_log("write_tray_icon_file: GetObject(hbmColor) failed");
    if (info.hbmColor)
      DeleteObject((HGDIOBJ)info.hbmColor);
    if (info.hbmMask)
      DeleteObject((HGDIOBJ)info.hbmMask);
    return;
  }

  int width = bmp.bmWidth;
  int height = bmp.bmHeight;
  DWORD pixel_bytes = (DWORD)(width * height * 4);

  BITMAPINFO bi;
  bi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
  bi.bmiHeader.biWidth = width;
  bi.bmiHeader.biHeight = -height;
  bi.bmiHeader.biPlanes = 1;
  bi.bmiHeader.biBitCount = 32;
  bi.bmiHeader.biCompression = BI_RGB;
  bi.bmiHeader.biSizeImage = 0;
  bi.bmiHeader.biXPelsPerMeter = 0;
  bi.bmiHeader.biYPelsPerMeter = 0;
  bi.bmiHeader.biClrUsed = 0;
  bi.bmiHeader.biClrImportant = 0;

  BYTE *pixels = (BYTE *)malloc(pixel_bytes);
  if (!pixels) {
    if (info.hbmColor)
      DeleteObject((HGDIOBJ)info.hbmColor);
    if (info.hbmMask)
      DeleteObject((HGDIOBJ)info.hbmMask);
    return;
  }

  HDC hdc = GetDC(NULL);
  int got =
      GetDIBits(hdc, info.hbmColor, 0, height, pixels, &bi, DIB_RGB_COLORS);
  ReleaseDC(NULL, hdc);
  if (info.hbmColor)
    DeleteObject((HGDIOBJ)info.hbmColor);
  if (info.hbmMask)
    DeleteObject((HGDIOBJ)info.hbmMask);

  if (got == 0) {
    debug_log("write_tray_icon_file: GetDIBits failed");
    free(pixels);
    return;
  }

  char tooltip[128];
  int tooltip_len = WideCharToMultiByte(CP_UTF8, 0, nid->szTip, -1, tooltip,
                                        sizeof(tooltip), NULL, NULL);
  if (tooltip_len > 0) {
    tooltip_len -= 1;
  } else {
    tooltip_len = 0;
  }

  HANDLE h = CreateFileW(path, GENERIC_WRITE, FILE_SHARE_READ, NULL,
                         CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
  if (h == INVALID_HANDLE_VALUE) {
    debug_log("write_tray_icon_file: failed to open tray icon file for write");
    free(pixels);
    return;
  }

  UINT32 magic = 0x59415254;
  UINT32 version = 1;
  UINT64 hwnd64 = (UINT64)(ULONG_PTR)nid->hWnd;
  UINT32 uid = nid->uID;
  UINT32 callback_msg = nid->uCallbackMessage;
  INT32 w32 = width, h32 = height;
  UINT32 tlen = (UINT32)tooltip_len;

  DWORD written;
  WriteFile(h, &magic, 4, &written, NULL);
  WriteFile(h, &version, 4, &written, NULL);
  WriteFile(h, &hwnd64, 8, &written, NULL);
  WriteFile(h, &uid, 4, &written, NULL);
  WriteFile(h, &callback_msg, 4, &written, NULL);
  WriteFile(h, &w32, 4, &written, NULL);
  WriteFile(h, &h32, 4, &written, NULL);
  WriteFile(h, &tlen, 4, &written, NULL);
  if (tooltip_len > 0) {
    WriteFile(h, tooltip, (DWORD)tooltip_len, &written, NULL);
  }
  WriteFile(h, pixels, pixel_bytes, &written, NULL);
  CloseHandle(h);
  free(pixels);

  g_tray_hwnd = hwnd64;
  g_tray_uid = uid;
  g_tray_callback_msg = callback_msg;

  char logbuf[96];
  wsprintfA(
      logbuf,
      "write_tray_icon_file: wrote %dx%d icon, uid=%lu callback_msg=0x%lx",
      width, height, (unsigned long)uid, (unsigned long)callback_msg);
  debug_log(logbuf);
}

void post_tray_click(BOOL is_right_click) {
  char logbuf[128];

  if (!g_tray_hwnd) {
    return;
  }

  if (g_tray_icon_version_4) {
    POINT pt = {0, 0};
    GetCursorPos(&pt);
    WPARAM wp = MAKEWPARAM((WORD)pt.x, (WORD)pt.y);
    WORD notify_code = is_right_click ? WM_CONTEXTMENU : NIN_SELECT;
    LPARAM lp = MAKELPARAM(notify_code, (WORD)g_tray_uid);
    PostMessageW((HWND)(ULONG_PTR)g_tray_hwnd, g_tray_callback_msg, wp, lp);

    wsprintfA(logbuf,
              "post_tray_click: V4 format, x=%d y=%d notify=0x%x uid=%lu", pt.x,
              pt.y, notify_code, (unsigned long)g_tray_uid);
    debug_log(logbuf);
  } else {
    UINT msg = is_right_click ? WM_RBUTTONUP : WM_LBUTTONUP;
    PostMessageW((HWND)(ULONG_PTR)g_tray_hwnd, g_tray_callback_msg,
                 (WPARAM)g_tray_uid, (LPARAM)msg);

    wsprintfA(logbuf, "post_tray_click: legacy format, msg=0x%x uid=%lu",
              (unsigned int)msg, (unsigned long)g_tray_uid);
    debug_log(logbuf);
  }
}

typedef BOOL(WINAPI *ShellNotifyIconW_t)(DWORD, PNOTIFYICONDATAW);
static ShellNotifyIconW_t real_Shell_NotifyIconW;

static BOOL WINAPI fake_Shell_NotifyIconW(DWORD dwMessage,
                                          PNOTIFYICONDATAW nid) {
  BOOL r =
      real_Shell_NotifyIconW ? real_Shell_NotifyIconW(dwMessage, nid) : FALSE;

  char logbuf[128];
  wsprintfA(logbuf,
            "fake_Shell_NotifyIconW: dwMessage=%lu uVersion=%lu uFlags=0x%lx "
            "uid=%lu -> %d",
            (unsigned long)dwMessage, nid ? (unsigned long)nid->uVersion : 0,
            nid ? (unsigned long)nid->uFlags : 0,
            nid ? (unsigned long)nid->uID : 0, r);
  debug_log(logbuf);

  if (dwMessage == NIM_SETVERSION && nid) {
    InterlockedExchange((LONG volatile *)&g_tray_icon_version_4,
                        nid->uVersion >= NOTIFYICON_VERSION_4 ? 1 : 0);
  }

  if ((dwMessage == NIM_ADD || dwMessage == NIM_MODIFY) && nid) {
    write_tray_icon_file(nid);
  }

  return r;
}

void install_tray_hooks(void) {
  FARPROC origTray =
      patch_iat(GetModuleHandleW(NULL), "SHELL32.dll", "Shell_NotifyIconW",
                (FARPROC)(void *)fake_Shell_NotifyIconW);
  real_Shell_NotifyIconW = (ShellNotifyIconW_t)(void *)origTray;
  debug_log(
      origTray ? "install_progman_hook: patch_iat Shell_NotifyIconW OK"
               : "install_progman_hook: patch_iat Shell_NotifyIconW NOT FOUND");
}
