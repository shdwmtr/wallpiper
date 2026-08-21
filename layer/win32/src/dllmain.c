#include "dwm_exports.h"
#include "ipc.h"
#include "progman.h"
#include "util.h"
#include "win32/imports.h"
#include "win32/windows.h"

#ifdef __TINYC__
BOOL WINAPI DllMain(HINSTANCE inst, DWORD reason, LPVOID reserved);

BOOL WINAPI _dllstart(HINSTANCE inst, DWORD reason, LPVOID reserved) {
  return DllMain(inst, reason, reserved);
}
#endif

BOOL WINAPI DllMain(HINSTANCE inst, DWORD reason, LPVOID reserved) {
  (void)inst;
  (void)reserved;

  if (reason == DLL_PROCESS_ATTACH) {
    win32_resolve_all();
    debug_log("DllMain DLL_PROCESS_ATTACH");
    DisableThreadLibraryCalls(inst);
    load_real_dwmapi();
    install_progman_hook();
  }

  return TRUE;
}
