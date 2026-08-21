#include "cursor.h"
#include "util.h"

#define SNARE_STATIC
#define SNARE_IMPLEMENTATION
#include "libsnare.h"

UINT64 g_tray_hwnd;

static DWORD WINAPI cursor_forward_thread(LPVOID param) {
  (void)param;
  debug_log("cursor_forward_thread: started");

  for (;;) {
    Sleep(16);

    HWND hwnd = (HWND)(ULONG_PTR)g_tray_hwnd;
    if (!hwnd) {
      continue;
    }

    POINT pt;
    if (!GetCursorPos(&pt)) {
      continue;
    }

    POINT client = pt;
    ScreenToClient(hwnd, &client);

    PostMessageW(hwnd, WM_MOUSEMOVE, 0,
                 MAKELPARAM((WORD)client.x, (WORD)client.y));
  }

  return 0;
}

static BOOL get_cursor_pos_file_path(char *out, DWORD out_len) {
  char unix_path[MAX_PATH];
  DWORD len = GetEnvironmentVariableA("WALLPIPER_CURSOR_POS_FILE", unix_path,
                                      sizeof(unix_path));
  if (len == 0 || len >= sizeof(unix_path)) {
    return FALSE;
  }
  int n = wsprintfA(out, "Z:%s", unix_path);
  return n > 0 && (DWORD)n < out_len;
}

static volatile LONGLONG g_forwarded_cursor;
static volatile LONG g_have_forwarded_cursor;

static void store_forwarded_cursor(INT32 x, INT32 y) {
  UINT64 packed = ((UINT64)(UINT32)x << 32) | (UINT32)y;
  InterlockedExchange64((LONGLONG volatile *)&g_forwarded_cursor,
                        (LONGLONG)packed);
  InterlockedExchange((LONG volatile *)&g_have_forwarded_cursor, 1);
}

static BOOL load_forwarded_cursor(LONG *x, LONG *y) {
  if (!InterlockedCompareExchange((LONG volatile *)&g_have_forwarded_cursor, 0,
                                  0)) {
    return FALSE;
  }
  UINT64 packed = (UINT64)InterlockedCompareExchange64(
      (LONGLONG volatile *)&g_forwarded_cursor, 0, 0);
  *x = (LONG)(INT32)(packed >> 32);
  *y = (LONG)(INT32)(packed & 0xFFFFFFFF);
  return TRUE;
}

typedef BOOL(WINAPI *GetCursorPos_t)(LPPOINT);
static GetCursorPos_t real_GetCursorPos;

static BOOL WINAPI fake_GetCursorPos(LPPOINT lpPoint) {
  if (!lpPoint) {
    return real_GetCursorPos ? real_GetCursorPos(lpPoint) : FALSE;
  }

  LONG x, y;
  if (load_forwarded_cursor(&x, &y)) {
    lpPoint->x = x;
    lpPoint->y = y;
    return TRUE;
  }

  return real_GetCursorPos ? real_GetCursorPos(lpPoint) : FALSE;
}

static snare_inline_t g_get_cursor_pos_hook;

static void install_get_cursor_pos_hook(void) {
  HMODULE user32 = GetModuleHandleW(L"user32.dll");
  if (!user32) {
    user32 = LoadLibraryW(L"user32.dll");
  }
  if (!user32) {
    debug_log("install_get_cursor_pos_hook: could not resolve user32.dll");
    return;
  }

  void *target = (void *)GetProcAddress(user32, "GetCursorPos");
  if (!target) {
    debug_log(
        "install_get_cursor_pos_hook: GetProcAddress(GetCursorPos) failed");
    return;
  }

  snare_inline_t hook = snare_inline_new(target, (void *)fake_GetCursorPos);
  if (!hook) {
    debug_log("install_get_cursor_pos_hook: snare_inline_new failed");
    return;
  }

  if (snare_inline_install(hook) != 0) {
    debug_log("install_get_cursor_pos_hook: snare_inline_install failed");
    snare_inline_free(hook);
    return;
  }

  real_GetCursorPos = (GetCursorPos_t)snare_inline_get_trampoline(hook);
  g_get_cursor_pos_hook = hook;
  debug_log("install_get_cursor_pos_hook: inline hook installed on "
            "user32.dll!GetCursorPos");
}

static DWORD WINAPI cursor_ipc_poll_thread(LPVOID param) {
  (void)param;

  char path[MAX_PATH + 2];
  if (!get_cursor_pos_file_path(path, sizeof(path))) {
    debug_log("cursor_ipc_poll_thread: WALLPIPER_CURSOR_POS_FILE not set");
    return 0;
  }

  debug_log("cursor_ipc_poll_thread: started");

  for (;;) {
    Sleep(16);

    HANDLE h =
        CreateFileA(path, GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE,
                    NULL, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE) {
      continue;
    }

    INT32 payload[2];
    DWORD read_bytes = 0;
    BOOL ok = ReadFile(h, payload, sizeof(payload), &read_bytes, NULL);
    CloseHandle(h);

    if (!ok || read_bytes != sizeof(payload)) {
      continue;
    }

    store_forwarded_cursor(payload[0], payload[1]);
  }

  return 0;
}

void install_cursor_hooks(void) {
  install_get_cursor_pos_hook();

  HANDLE cursor_thread =
      CreateThread(NULL, 0, cursor_forward_thread, NULL, 0, NULL);
  if (cursor_thread) {
    CloseHandle(cursor_thread);
  }

  HANDLE cursor_ipc_thread =
      CreateThread(NULL, 0, cursor_ipc_poll_thread, NULL, 0, NULL);
  if (cursor_ipc_thread) {
    CloseHandle(cursor_ipc_thread);
  }
}
