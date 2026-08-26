/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2026 Ethan Alexander
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#include "bootstrap.h"
#include "structs.h"

typedef struct {
  WORD Length;
  WORD MaximumLength;
  LPWSTR Buffer;
} unicode_string_min_t;

static PVOID read_peb(void) {
  PVOID peb;
  __asm__ volatile("movq %%gs:0x60, %0" : "=r"(peb));
  return peb;
}

static int wide_name_ieq_ascii(const WCHAR *w, WORD w_len_bytes,
                               const char *ascii_upper) {
  WORD w_len_chars = (WORD)(w_len_bytes / 2);
  WORD i = 0;
  for (; ascii_upper[i]; i++) {
    if (i >= w_len_chars)
      return 0;
    WCHAR c = w[i];
    if (c >= 'a' && c <= 'z')
      c = (WCHAR)(c - 32);
    if (c != (WCHAR)(unsigned char)ascii_upper[i])
      return 0;
  }
  return i == w_len_chars;
}

PVOID win32_bootstrap_find_kernel32(void) {
  unsigned char *peb = (unsigned char *)read_peb();
  unsigned char *ldr = *(unsigned char **)(peb + 0x18);
  unsigned char *list_head = ldr + 0x20;
  unsigned char *node = *(unsigned char **)list_head;

  for (; node != list_head; node = *(unsigned char **)node) {
    unsigned char *entry = node - 0x10;
    unicode_string_min_t *base_name = (unicode_string_min_t *)(entry + 0x58);
    if (base_name->Buffer &&
        wide_name_ieq_ascii(base_name->Buffer, base_name->Length,
                            "KERNEL32.DLL")) {
      return *(PVOID *)(entry + 0x30);
    }
  }
  return NULL;
}

FARPROC win32_bootstrap_find_export(PVOID module_base, const char *name) {
  unsigned char *base = (unsigned char *)module_base;
  if (!base)
    return NULL;

  IMAGE_DOS_HEADER *dos = (IMAGE_DOS_HEADER *)base;
  IMAGE_NT_HEADERS64 *nt = (IMAGE_NT_HEADERS64 *)(base + dos->e_lfanew);
  IMAGE_DATA_DIRECTORY exp_dir =
      nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT];
  if (exp_dir.VirtualAddress == 0)
    return NULL;

  IMAGE_EXPORT_DIRECTORY *exp =
      (IMAGE_EXPORT_DIRECTORY *)(base + exp_dir.VirtualAddress);
  DWORD *names = (DWORD *)(base + exp->AddressOfNames);
  WORD *ordinals = (WORD *)(base + exp->AddressOfNameOrdinals);
  DWORD *functions = (DWORD *)(base + exp->AddressOfFunctions);

  for (DWORD i = 0; i < exp->NumberOfNames; i++) {
    const char *export_name = (const char *)(base + names[i]);
    const char *a = export_name;
    const char *b = name;
    while (*a && *a == *b) {
      a++;
      b++;
    }
    if (*a == *b) {
      WORD ord = ordinals[i];
      return (FARPROC)(base + functions[ord]);
    }
  }
  return NULL;
}
