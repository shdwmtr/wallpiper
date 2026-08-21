#include "imports.h"
#include "bootstrap.h"
// #include "util.h"
// #include "libc.h"

#define WIN32_IMPORT(dll, ret, name, params) ret(WINAPI *name) params;
#include "import_table.def"
#undef WIN32_IMPORT

typedef HMODULE(WINAPI *raw_load_library_a_t)(LPCSTR);
typedef FARPROC(WINAPI *raw_get_proc_address_t)(HMODULE, LPCSTR);

typedef struct {
  const char *dll;
  const char *name;
  void **slot;
} win32_import_entry_t;

#define WIN32_IMPORT(dll, ret, name, params)                                   \
  {#dll ".dll", #name, (void **)&name},
static const win32_import_entry_t g_import_table[] = {
#include "import_table.def"
};
#undef WIN32_IMPORT

#define IMPORT_COUNT (sizeof(g_import_table) / sizeof(g_import_table[0]))

void win32_resolve_all(void) {
  PVOID kernel32_base = win32_bootstrap_find_kernel32();
  raw_get_proc_address_t raw_gpa =
      (raw_get_proc_address_t)win32_bootstrap_find_export(kernel32_base,
                                                          "GetProcAddress");
  raw_load_library_a_t raw_lla =
      (raw_load_library_a_t)win32_bootstrap_find_export(kernel32_base,
                                                        "LoadLibraryA");

  unsigned failures = 0;
  const win32_import_entry_t *failed[16];

  if (!raw_gpa || !raw_lla) {
    return;
  }

  for (unsigned i = 0; i < IMPORT_COUNT; i++) {
    const win32_import_entry_t *entry = &g_import_table[i];
    HMODULE module = raw_lla(entry->dll);
    if (!module) {
      if (failures < 16)
        failed[failures] = entry;
      failures++;
      continue;
    }
    FARPROC addr = raw_gpa(module, entry->name);
    if (!addr) {
      if (failures < 16)
        failed[failures] = entry;
      failures++;
      continue;
    }
    *entry->slot = (void *)addr;
  }

  // if (failures > 0) {
  //     char logbuf[64];
  //     snprintf(logbuf, sizeof(logbuf), "win32_resolve_all: %u/%u symbols
  //     failed to resolve", failures, (unsigned)IMPORT_COUNT);
  //     debug_log(logbuf);
  //     for (unsigned i = 0; i < failures && i < 16; i++) {
  //         snprintf(logbuf, sizeof(logbuf), "  failed: %s!%s", failed[i]->dll,
  //         failed[i]->name); debug_log(logbuf);
  //     }
  // }
}
