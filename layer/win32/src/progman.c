#include "progman.h"
#include "cursor.h"
#include "ipc_triggers.h"
#include "menu.h"
#include "pe_iat.h"
#include "spawn.h"
#include "tray.h"
#include "util.h"
#include "waitobj.h"

HWND g_fake_empty_workerw;
HWND g_fake_icon_workerw;
static HWND g_fake_progman;
static HWND g_fake_defview;
typedef HWND(WINAPI *FindWindowW_t)(LPCWSTR, LPCWSTR);
static FindWindowW_t real_FindWindowW;

static void ensure_fake_progman_family(void) {
  if (g_fake_progman && IsWindow(g_fake_progman) && g_fake_empty_workerw &&
      IsWindow(g_fake_empty_workerw)) {
    return;
  }

  HINSTANCE hinst = GetModuleHandleW(NULL);
  WNDCLASSEXW wc = {0};
  wc.cbSize = sizeof(wc);
  wc.lpfnWndProc = DefWindowProcW;
  wc.hInstance = hinst;

  wc.lpszClassName = L"Progman";
  RegisterClassExW(&wc);
  wc.lpszClassName = L"WorkerW";
  RegisterClassExW(&wc);
  wc.lpszClassName = L"SHELLDLL_DefView";
  RegisterClassExW(&wc);

  g_fake_progman = CreateWindowExW(0, L"Progman", L"Program Manager", WS_POPUP,
                                   0, 0, 0, 0, NULL, NULL, hinst, NULL);
  g_fake_icon_workerw = CreateWindowExW(0, L"WorkerW", L"", WS_POPUP, 0, 0, 0,
                                        0, NULL, NULL, hinst, NULL);
  g_fake_defview =
      CreateWindowExW(0, L"SHELLDLL_DefView", L"", WS_CHILD, 0, 0, 0, 0,
                      g_fake_icon_workerw, NULL, hinst, NULL);

  int vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
  int vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
  int sw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
  int sh = GetSystemMetrics(SM_CYVIRTUALSCREEN);
  if (sw <= 0)
    sw = 1920;
  if (sh <= 0)
    sh = 1080;

  g_fake_empty_workerw = CreateWindowExW(0, L"WorkerW", L"", WS_POPUP, vx, vy,
                                         sw, sh, NULL, NULL, hinst, NULL);
}

void reassert_workerw_zorder(void) {
  if (g_fake_empty_workerw && g_fake_icon_workerw) {
    SetWindowPos(g_fake_empty_workerw, g_fake_icon_workerw, 0, 0, 0, 0,
                 SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
  }
}

static HWND WINAPI fake_FindWindowW(LPCWSTR lpClassName, LPCWSTR lpWindowName) {
  if (lpClassName && !IS_INTRESOURCE(lpClassName) && !lpWindowName &&
      lstrcmpW(lpClassName, L"Progman") == 0) {
    ensure_fake_progman_family();
    reassert_workerw_zorder();
    char logbuf[80];
    wsprintfA(logbuf, "fake_FindWindowW: faking Progman, returning %p",
              g_fake_progman);
    debug_log(logbuf);
    return g_fake_progman;
  }

  return real_FindWindowW ? real_FindWindowW(lpClassName, lpWindowName) : NULL;
}

void install_progman_hook(void) {
  if (!running_as_wallpaper64()) {
    return;
  }

  FARPROC orig = patch_iat(GetModuleHandleW(NULL), "USER32.dll", "FindWindowW",
                           (FARPROC)fake_FindWindowW);
  real_FindWindowW = (FindWindowW_t)(void *)orig;
  debug_log(orig ? "install_progman_hook: patch_iat FindWindowW OK"
                 : "install_progman_hook: patch_iat FindWindowW NOT FOUND");

  install_tray_hooks();
  install_menu_hooks();
  install_wait_hooks();
  install_spawn_hooks();

  HANDLE menu_thread =
      CreateThread(NULL, 0, file_trigger_watcher_thread, NULL, 0, NULL);
  if (menu_thread) {
    CloseHandle(menu_thread);
  }

  install_cursor_hooks();
}
