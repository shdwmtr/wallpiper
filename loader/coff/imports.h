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

#ifndef WP_WIN32_IMPORTS_H
#define WP_WIN32_IMPORTS_H

#include "structs.h"
#include "types.h"

#define WIN32_IMPORT(dll, ret, name, params) extern ret(WINAPI *name) params;
#include "import_table.def"
#undef WIN32_IMPORT

void win32_resolve_all(void);

static inline LONG wp_InterlockedExchange(LONG volatile *Target, LONG Value) {
  __asm__ volatile("lock xchgl %0, %1"
                   : "+r"(Value), "+m"(*Target)
                   :
                   : "memory");
  return Value;
}

static inline LONG wp_InterlockedCompareExchange(LONG volatile *Destination,
                                                 LONG Exchange,
                                                 LONG Comparand) {
  __asm__ volatile("lock cmpxchgl %2, %1"
                   : "+a"(Comparand), "+m"(*Destination)
                   : "r"(Exchange)
                   : "memory");
  return Comparand;
}

static inline LONGLONG wp_InterlockedExchange64(LONGLONG volatile *Target,
                                                LONGLONG Value) {
  __asm__ volatile("lock xchgq %0, %1"
                   : "+r"(Value), "+m"(*Target)
                   :
                   : "memory");
  return Value;
}

static inline LONGLONG
wp_InterlockedCompareExchange64(LONGLONG volatile *Destination,
                                LONGLONG Exchange, LONGLONG Comparand) {
  __asm__ volatile("lock cmpxchgq %2, %1"
                   : "+a"(Comparand), "+m"(*Destination)
                   : "r"(Exchange)
                   : "memory");
  return Comparand;
}

#define InterlockedExchange wp_InterlockedExchange
#define InterlockedCompareExchange wp_InterlockedCompareExchange
#define InterlockedExchange64 wp_InterlockedExchange64
#define InterlockedCompareExchange64 wp_InterlockedCompareExchange64

#endif
