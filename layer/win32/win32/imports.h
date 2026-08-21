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
