#include "menu.h"
#include "pe_iat.h"
#include "util.h"
#include <string.h>

static size_t append_str(char *buf, size_t buf_cap, size_t pos, const char *s) {
  size_t len = (size_t)lstrlenA(s);
  if (pos + len >= buf_cap) {
    return pos;
  }
  memcpy(buf + pos, s, len);
  return pos + len;
}

static size_t serialize_menu(HMENU hMenu, int depth, char *buf, size_t buf_cap,
                             size_t pos) {
  if (!hMenu || depth > 4) {
    return pos;
  }

  WCHAR *textbuf = (WCHAR *)malloc(256 * sizeof(WCHAR));
  MENUITEMINFOW *mii = (MENUITEMINFOW *)malloc(sizeof(MENUITEMINFOW));
  char *textutf8 = (char *)malloc(256);
  char *line = (char *)malloc(400);
  if (!textbuf || !mii || !textutf8 || !line) {
    free(textbuf);
    free(mii);
    free(textutf8);
    free(line);
    return pos;
  }

  int count = GetMenuItemCount(hMenu);
  for (int i = 0; i < count; i++) {
    textbuf[0] = L'\0';

    mii->cbSize = sizeof(*mii);
    mii->fMask = MIIM_ID | MIIM_STATE | MIIM_SUBMENU | MIIM_FTYPE | MIIM_STRING;
    mii->dwTypeData = textbuf;
    mii->cch = 256;

    if (!GetMenuItemInfoW(hMenu, (UINT)i, TRUE, mii)) {
      continue;
    }

    int n =
        WideCharToMultiByte(CP_UTF8, 0, textbuf, -1, textutf8, 256, NULL, NULL);
    if (n <= 0) {
      textutf8[0] = '\0';
    }
    for (char *p = textutf8; *p; p++) {
      if (*p == '\t' || *p == '\n' || *p == '\r') {
        *p = ' ';
      }
    }

    wsprintfA(line, "%d\t%lu\t0x%lx\t0x%lx\t%s\n", depth,
              (unsigned long)mii->wID, (unsigned long)mii->fState,
              (unsigned long)mii->fType, textutf8);
    pos = append_str(buf, buf_cap, pos, line);

    if (mii->hSubMenu) {
      pos = serialize_menu(mii->hSubMenu, depth + 1, buf, buf_cap, pos);
    }
  }

  free(textbuf);
  free(mii);
  free(textutf8);
  free(line);
  return pos;
}

UINT64 g_menu_owner_hwnd;
static HMENU g_tray_root_menu;

static void dump_menu_to_file(HMENU hMenu) {
  WCHAR path[MAX_PATH];
  if (!GetEnvironmentVariableW(L"WALLPIPER_MENU_FILE", path, MAX_PATH)) {
    return;
  }
  WCHAR tmp_path[MAX_PATH + 4];
  wsprintfW(tmp_path, L"%s.tmp", path);

  char *buf = (char *)malloc(16384);
  if (!buf) {
    return;
  }
  size_t pos = serialize_menu(hMenu, 0, buf, 16384, 0);

  HANDLE h = CreateFileW(tmp_path, GENERIC_WRITE, FILE_SHARE_READ, NULL,
                         CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
  if (h == INVALID_HANDLE_VALUE) {
    free(buf);
    return;
  }
  DWORD written;
  WriteFile(h, buf, (DWORD)pos, &written, NULL);
  CloseHandle(h);
  free(buf);

  MoveFileExW(tmp_path, path, MOVEFILE_REPLACE_EXISTING);
}

typedef BOOL(WINAPI *TrackPopupMenu_t)(HMENU, UINT, int, int, int, HWND,
                                       const RECT *);
typedef BOOL(WINAPI *TrackPopupMenuEx_t)(HMENU, UINT, int, int, HWND, LPVOID);
static TrackPopupMenu_t real_TrackPopupMenu;
static TrackPopupMenuEx_t real_TrackPopupMenuEx;

static BOOL WINAPI fake_TrackPopupMenu(HMENU hMenu, UINT uFlags, int x, int y,
                                       int nReserved, HWND hWnd,
                                       const RECT *prcRect) {
  volatile char stack_margin[512];
  stack_margin[0] = 0;
  stack_margin[511] = 0;

  (void)x;
  (void)y;
  (void)nReserved;
  (void)prcRect;

  g_menu_owner_hwnd = (UINT64)(ULONG_PTR)hWnd;
  dump_menu_to_file(hMenu);

  char logbuf[128];
  wsprintfA(logbuf,
            "fake_TrackPopupMenu: hMenu=%p hwnd=%p flags=0x%lx returncmd=%d, "
            "suppressed native menu (dumped to WALLPIPER_MENU_FILE)",
            hMenu, hWnd, (unsigned long)uFlags, (uFlags & TPM_RETURNCMD) != 0);
  debug_log(logbuf);

  return 0;
}

static BOOL WINAPI fake_TrackPopupMenuEx(HMENU hMenu, UINT uFlags, int x, int y,
                                         HWND hWnd, LPVOID lptpm) {
  volatile char stack_margin[512];
  stack_margin[0] = 0;
  stack_margin[511] = 0;

  (void)x;
  (void)y;
  (void)lptpm;

  g_menu_owner_hwnd = (UINT64)(ULONG_PTR)hWnd;
  dump_menu_to_file(hMenu);

  char logbuf[128];
  wsprintfA(logbuf,
            "fake_TrackPopupMenuEx: hMenu=%p hwnd=%p flags=0x%lx returncmd=%d, "
            "suppressed native menu (dumped to WALLPIPER_MENU_FILE)",
            hMenu, hWnd, (unsigned long)uFlags, (uFlags & TPM_RETURNCMD) != 0);
  debug_log(logbuf);

  return 0;
}

typedef HMENU(WINAPI *CreatePopupMenu_t)(void);
typedef BOOL(WINAPI *AppendMenuW_t)(HMENU, UINT, UINT_PTR, LPCWSTR);
typedef BOOL(WINAPI *InsertMenuItemW_t)(HMENU, UINT, BOOL, LPCMENUITEMINFOW);
static CreatePopupMenu_t real_CreatePopupMenu;
static AppendMenuW_t real_AppendMenuW;
static InsertMenuItemW_t real_InsertMenuItemW;

static HMENU WINAPI fake_CreatePopupMenu(void) {
  HMENU h = real_CreatePopupMenu ? real_CreatePopupMenu() : NULL;
  char logbuf[64];
  wsprintfA(logbuf, "CreatePopupMenu: -> %p", h);
  menu_build_log(logbuf);
  return h;
}

static BOOL WINAPI fake_AppendMenuW(HMENU hMenu, UINT uFlags,
                                    UINT_PTR uIDNewItem, LPCWSTR lpNewItem) {
  BOOL r = real_AppendMenuW
               ? real_AppendMenuW(hMenu, uFlags, uIDNewItem, lpNewItem)
               : FALSE;

  char textbuf[128] = "";
  if (lpNewItem && !(uFlags & MF_BITMAP) && !(uFlags & MF_OWNERDRAW) &&
      (ULONG_PTR)lpNewItem > 0xFFFF) {
    int n = WideCharToMultiByte(CP_UTF8, 0, lpNewItem, -1, textbuf,
                                sizeof(textbuf), NULL, NULL);
    if (n <= 0) {
      textbuf[0] = '\0';
    }
  }

  char logbuf[220];
  wsprintfA(logbuf, "AppendMenuW: hMenu=%p flags=0x%lx id=%lu text=%s -> %d",
            hMenu, (unsigned long)uFlags, (unsigned long)uIDNewItem, textbuf,
            r);
  menu_build_log(logbuf);
  return r;
}

static BOOL WINAPI fake_InsertMenuItemW(HMENU hMenu, UINT item,
                                        BOOL fByPosition,
                                        LPCMENUITEMINFOW lpmii) {
  BOOL r = real_InsertMenuItemW
               ? real_InsertMenuItemW(hMenu, item, fByPosition, lpmii)
               : FALSE;

  char textbuf[128] = "";
  if (lpmii && (lpmii->fMask & MIIM_STRING) && lpmii->dwTypeData) {
    int n = WideCharToMultiByte(CP_UTF8, 0, lpmii->dwTypeData, -1, textbuf,
                                sizeof(textbuf), NULL, NULL);
    if (n <= 0) {
      textbuf[0] = '\0';
    }
  }
  UINT id = (lpmii && (lpmii->fMask & MIIM_ID)) ? lpmii->wID : 0;

  char logbuf[220];
  wsprintfA(logbuf,
            "InsertMenuItemW: hMenu=%p item=%u byPos=%d id=%u text=%s -> %d",
            hMenu, item, fByPosition, id, textbuf, r);
  menu_build_log(logbuf);

  if (r && g_tray_root_menu) {
    dump_menu_to_file(g_tray_root_menu);
  }
  return r;
}

typedef HMENU(WINAPI *LoadMenuW_t)(HINSTANCE, LPCWSTR);
typedef HMENU(WINAPI *GetSubMenu_t)(HMENU, int);
typedef BOOL(WINAPI *InsertMenuW_t)(HMENU, UINT, UINT, UINT_PTR, LPCWSTR);
typedef BOOL(WINAPI *SetMenuItemInfoW_t)(HMENU, UINT, BOOL, LPCMENUITEMINFOW);
typedef BOOL(WINAPI *DestroyMenu_t)(HMENU);
static LoadMenuW_t real_LoadMenuW;
static GetSubMenu_t real_GetSubMenu;
static InsertMenuW_t real_InsertMenuW;
static SetMenuItemInfoW_t real_SetMenuItemInfoW;
static DestroyMenu_t real_DestroyMenu;

static HMENU g_pending_menu_wrapper;

static HMENU WINAPI fake_LoadMenuW(HINSTANCE hInstance, LPCWSTR lpMenuName) {
  HMENU h = real_LoadMenuW ? real_LoadMenuW(hInstance, lpMenuName) : NULL;

  char logbuf[64];
  wsprintfA(logbuf, "LoadMenuW: -> %p", h);
  menu_build_log(logbuf);

  if (h) {
    g_pending_menu_wrapper = h;
  }
  return h;
}

static HMENU WINAPI fake_GetSubMenu(HMENU hMenu, int nPos) {
  HMENU h = real_GetSubMenu ? real_GetSubMenu(hMenu, nPos) : NULL;

  char logbuf[80];
  wsprintfA(logbuf, "GetSubMenu: hMenu=%p pos=%d -> %p", hMenu, nPos, h);
  menu_build_log(logbuf);

  if (h && hMenu == g_pending_menu_wrapper) {
    g_tray_root_menu = h;
    dump_menu_to_file(h);
  }
  return h;
}

static BOOL WINAPI fake_InsertMenuW(HMENU hMenu, UINT position, UINT flags,
                                    UINT_PTR idNewItem, LPCWSTR lpNewItem) {
  BOOL r = real_InsertMenuW
               ? real_InsertMenuW(hMenu, position, flags, idNewItem, lpNewItem)
               : FALSE;

  char textbuf[128] = "";
  if (lpNewItem && !(flags & MF_BITMAP) && !(flags & MF_OWNERDRAW) &&
      (ULONG_PTR)lpNewItem > 0xFFFF) {
    int n = WideCharToMultiByte(CP_UTF8, 0, lpNewItem, -1, textbuf,
                                sizeof(textbuf), NULL, NULL);
    if (n <= 0) {
      textbuf[0] = '\0';
    }
  }

  char logbuf[220];
  wsprintfA(logbuf,
            "InsertMenuW: hMenu=%p position=%u flags=0x%lx id=%lu text=%s "
            "-> %d",
            hMenu, position, (unsigned long)flags, (unsigned long)idNewItem,
            textbuf, r);
  menu_build_log(logbuf);

  if (r && g_tray_root_menu) {
    dump_menu_to_file(g_tray_root_menu);
  }
  return r;
}

static BOOL WINAPI fake_SetMenuItemInfoW(HMENU hMenu, UINT item,
                                         BOOL fByPosition,
                                         LPCMENUITEMINFOW lpmii) {
  BOOL r = real_SetMenuItemInfoW
               ? real_SetMenuItemInfoW(hMenu, item, fByPosition, lpmii)
               : FALSE;

  char logbuf[128];
  wsprintfA(logbuf, "SetMenuItemInfoW: hMenu=%p item=%u byPos=%d -> %d", hMenu,
            item, fByPosition, r);
  menu_build_log(logbuf);

  if (r && g_tray_root_menu) {
    dump_menu_to_file(g_tray_root_menu);
  }
  return r;
}

static BOOL WINAPI fake_DestroyMenu(HMENU hMenu) {
  BOOL r = real_DestroyMenu ? real_DestroyMenu(hMenu) : FALSE;

  char logbuf[64];
  wsprintfA(logbuf, "DestroyMenu: hMenu=%p -> %d", hMenu, r);
  menu_build_log(logbuf);

  if (r && hMenu == g_tray_root_menu) {
    g_tray_root_menu = NULL;
  } else if (r && g_tray_root_menu) {
    dump_menu_to_file(g_tray_root_menu);
  }
  return r;
}

void install_menu_hooks(void) {
  FARPROC origTPM =
      patch_iat(GetModuleHandleW(NULL), "USER32.dll", "TrackPopupMenu",
                (FARPROC)(void *)fake_TrackPopupMenu);
  real_TrackPopupMenu = (TrackPopupMenu_t)(void *)origTPM;
  debug_log(origTPM
                ? "install_progman_hook: patch_iat TrackPopupMenu OK"
                : "install_progman_hook: patch_iat TrackPopupMenu NOT FOUND");

  FARPROC origTPMEx =
      patch_iat(GetModuleHandleW(NULL), "USER32.dll", "TrackPopupMenuEx",
                (FARPROC)(void *)fake_TrackPopupMenuEx);
  real_TrackPopupMenuEx = (TrackPopupMenuEx_t)(void *)origTPMEx;
  debug_log(origTPMEx
                ? "install_progman_hook: patch_iat TrackPopupMenuEx OK"
                : "install_progman_hook: patch_iat TrackPopupMenuEx NOT FOUND");

  FARPROC origCPM = NULL;
  int nCPM =
      patch_iat_all_modules("USER32.dll", "CreatePopupMenu",
                            (FARPROC)(void *)fake_CreatePopupMenu, &origCPM);
  real_CreatePopupMenu = (CreatePopupMenu_t)(void *)origCPM;
  {
    char b[80];
    wsprintfA(b,
              "install_progman_hook: patch_iat_all_modules CreatePopupMenu "
              "patched=%d modules",
              nCPM);
    debug_log(b);
  }

  FARPROC origAMW = NULL;
  int nAMW = patch_iat_all_modules("USER32.dll", "AppendMenuW",
                                   (FARPROC)(void *)fake_AppendMenuW, &origAMW);
  real_AppendMenuW = (AppendMenuW_t)(void *)origAMW;
  {
    char b[80];
    wsprintfA(b,
              "install_progman_hook: patch_iat_all_modules AppendMenuW "
              "patched=%d modules",
              nAMW);
    debug_log(b);
  }

  FARPROC origIMIW = NULL;
  int nIMIW =
      patch_iat_all_modules("USER32.dll", "InsertMenuItemW",
                            (FARPROC)(void *)fake_InsertMenuItemW, &origIMIW);
  real_InsertMenuItemW = (InsertMenuItemW_t)(void *)origIMIW;
  {
    char b[80];
    wsprintfA(b,
              "install_progman_hook: patch_iat_all_modules InsertMenuItemW "
              "patched=%d modules",
              nIMIW);
    debug_log(b);
  }

  FARPROC origLMW = NULL;
  int nLMW = patch_iat_all_modules("USER32.dll", "LoadMenuW",
                                   (FARPROC)(void *)fake_LoadMenuW, &origLMW);
  real_LoadMenuW = (LoadMenuW_t)(void *)origLMW;
  {
    char b[80];
    wsprintfA(b,
              "install_progman_hook: patch_iat_all_modules LoadMenuW "
              "patched=%d modules",
              nLMW);
    debug_log(b);
  }

  FARPROC origGSM = NULL;
  int nGSM = patch_iat_all_modules("USER32.dll", "GetSubMenu",
                                   (FARPROC)(void *)fake_GetSubMenu, &origGSM);
  real_GetSubMenu = (GetSubMenu_t)(void *)origGSM;
  {
    char b[80];
    wsprintfA(b,
              "install_progman_hook: patch_iat_all_modules GetSubMenu "
              "patched=%d modules",
              nGSM);
    debug_log(b);
  }

  FARPROC origIMW = NULL;
  int nIMW = patch_iat_all_modules("USER32.dll", "InsertMenuW",
                                   (FARPROC)(void *)fake_InsertMenuW, &origIMW);
  real_InsertMenuW = (InsertMenuW_t)(void *)origIMW;
  {
    char b[80];
    wsprintfA(b,
              "install_progman_hook: patch_iat_all_modules InsertMenuW "
              "patched=%d modules",
              nIMW);
    debug_log(b);
  }

  FARPROC origSMIIW = NULL;
  int nSMIIW =
      patch_iat_all_modules("USER32.dll", "SetMenuItemInfoW",
                            (FARPROC)(void *)fake_SetMenuItemInfoW, &origSMIIW);
  real_SetMenuItemInfoW = (SetMenuItemInfoW_t)(void *)origSMIIW;
  {
    char b[80];
    wsprintfA(b,
              "install_progman_hook: patch_iat_all_modules SetMenuItemInfoW "
              "patched=%d modules",
              nSMIIW);
    debug_log(b);
  }

  FARPROC origDM = NULL;
  int nDM = patch_iat_all_modules("USER32.dll", "DestroyMenu",
                                  (FARPROC)(void *)fake_DestroyMenu, &origDM);
  real_DestroyMenu = (DestroyMenu_t)(void *)origDM;
  {
    char b[80];
    wsprintfA(b,
              "install_progman_hook: patch_iat_all_modules DestroyMenu "
              "patched=%d modules",
              nDM);
    debug_log(b);
  }
}
