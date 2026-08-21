#include "pe_iat.h"
#include "util.h"

FARPROC patch_iat(HMODULE image, const char *dll_name, const char *func_name,
                  FARPROC replacement) {
  PIMAGE_DOS_HEADER dos = (PIMAGE_DOS_HEADER)image;
  PIMAGE_NT_HEADERS nt = (PIMAGE_NT_HEADERS)((BYTE *)image + dos->e_lfanew);
  PIMAGE_DATA_DIRECTORY import_dir =
      &nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
  if (!import_dir->VirtualAddress) {
    return NULL;
  }

  PIMAGE_IMPORT_DESCRIPTOR desc =
      (PIMAGE_IMPORT_DESCRIPTOR)((BYTE *)image + import_dir->VirtualAddress);
  for (; desc->Name; desc++) {
    const char *name = (const char *)((BYTE *)image + desc->Name);
    if (lstrcmpiA(name, dll_name) != 0) {
      continue;
    }

    PIMAGE_THUNK_DATA orig_thunk =
        (PIMAGE_THUNK_DATA)((BYTE *)image + desc->OriginalFirstThunk);
    PIMAGE_THUNK_DATA iat_thunk =
        (PIMAGE_THUNK_DATA)((BYTE *)image + desc->FirstThunk);

    for (; orig_thunk->u1.AddressOfData; orig_thunk++, iat_thunk++) {
      if (IMAGE_SNAP_BY_ORDINAL(orig_thunk->u1.Ordinal)) {
        continue;
      }
      PIMAGE_IMPORT_BY_NAME import_by_name =
          (PIMAGE_IMPORT_BY_NAME)((BYTE *)image + orig_thunk->u1.AddressOfData);
      if (!names_equal((const char *)import_by_name->Name, func_name)) {
        continue;
      }

      FARPROC original = (FARPROC)(void *)iat_thunk->u1.Function;
      DWORD old_protect;
      if (VirtualProtect(&iat_thunk->u1.Function, sizeof(void *),
                         PAGE_READWRITE, &old_protect)) {
        iat_thunk->u1.Function = (ULONGLONG)(ULONG_PTR)replacement;
        VirtualProtect(&iat_thunk->u1.Function, sizeof(void *), old_protect,
                       &old_protect);
      }
      return original;
    }
    return NULL;
  }
  return NULL;
}

int patch_iat_all_modules(const char *dll_name, const char *func_name,
                          FARPROC replacement, FARPROC *out_original) {
  HANDLE snap =
      CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, GetCurrentProcessId());
  if (snap == INVALID_HANDLE_VALUE) {
    return 0;
  }

  MODULEENTRY32W me;
  me.dwSize = sizeof(me);
  int patched = 0;

  if (Module32FirstW(snap, &me)) {
    do {
      FARPROC original =
          patch_iat((HMODULE)me.hModule, dll_name, func_name, replacement);
      if (original) {
        patched++;
        if (out_original && !*out_original) {
          *out_original = original;
        }
      }
    } while (Module32NextW(snap, &me));
  }

  CloseHandle(snap);
  return patched;
}
