#include "libc.h"
#include "imports.h"

void *malloc(size_t size) { return HeapAlloc(GetProcessHeap(), 0, size); }

void *calloc(size_t count, size_t size) {
  return HeapAlloc(GetProcessHeap(), 0x00000008 /* HEAP_ZERO_MEMORY */,
                   count * size);
}

void *realloc(void *ptr, size_t size) {
  if (!ptr)
    return malloc(size);
  if (size == 0) {
    HeapFree(GetProcessHeap(), 0, ptr);
    return NULL;
  }
  return HeapReAlloc(GetProcessHeap(), 0, ptr, size);
}

void free(void *ptr) {
  if (ptr)
    HeapFree(GetProcessHeap(), 0, ptr);
}

void *memcpy(void *dst, const void *src, size_t n) {
  unsigned char *d = (unsigned char *)dst;
  const unsigned char *s = (const unsigned char *)src;
  for (size_t i = 0; i < n; i++)
    d[i] = s[i];
  return dst;
}

void *memmove(void *dst, const void *src, size_t n) {
  unsigned char *d = (unsigned char *)dst;
  const unsigned char *s = (const unsigned char *)src;
  if (d == s || n == 0)
    return dst;
  if (d < s) {
    for (size_t i = 0; i < n; i++)
      d[i] = s[i];
  } else {
    for (size_t i = n; i > 0; i--)
      d[i - 1] = s[i - 1];
  }
  return dst;
}

void *memset(void *dst, int c, size_t n) {
  unsigned char *d = (unsigned char *)dst;
  for (size_t i = 0; i < n; i++)
    d[i] = (unsigned char)c;
  return dst;
}

int memcmp(const void *a, const void *b, size_t n) {
  const unsigned char *pa = (const unsigned char *)a;
  const unsigned char *pb = (const unsigned char *)b;
  for (size_t i = 0; i < n; i++) {
    if (pa[i] != pb[i])
      return (int)pa[i] - (int)pb[i];
  }
  return 0;
}

size_t strlen(const char *s) {
  size_t n = 0;
  while (s[n])
    n++;
  return n;
}

int strcmp(const char *a, const char *b) {
  while (*a && *a == *b) {
    a++;
    b++;
  }
  return (unsigned char)*a - (unsigned char)*b;
}

int strncmp(const char *a, const char *b, size_t n) {
  for (size_t i = 0; i < n; i++) {
    unsigned char ca = (unsigned char)a[i];
    unsigned char cb = (unsigned char)b[i];
    if (ca != cb)
      return (int)ca - (int)cb;
    if (ca == 0)
      return 0;
  }
  return 0;
}

char *strcpy(char *dst, const char *src) {
  char *out = dst;
  while ((*dst++ = *src++) != 0) {
  }
  return out;
}

char *strchr(const char *s, int c) {
  for (; *s; s++) {
    if (*s == (char)c)
      return (char *)s;
  }
  return c == 0 ? (char *)s : NULL;
}

char *strstr(const char *haystack, const char *needle) {
  if (!*needle)
    return (char *)haystack;
  for (; *haystack; haystack++) {
    const char *h = haystack;
    const char *n = needle;
    while (*h && *n && *h == *n) {
      h++;
      n++;
    }
    if (!*n)
      return (char *)haystack;
  }
  return NULL;
}

int strcasecmp(const char *a, const char *b) {
  while (*a && *b) {
    char ca = *a;
    char cb = *b;
    if (ca >= 'a' && ca <= 'z')
      ca = (char)(ca - 32);
    if (cb >= 'a' && cb <= 'z')
      cb = (char)(cb - 32);
    if (ca != cb)
      return (unsigned char)ca - (unsigned char)cb;
    a++;
    b++;
  }
  return (unsigned char)*a - (unsigned char)*b;
}

static void append_char(char *buf, size_t cap, size_t *pos, char c) {
  if (*pos + 1 < cap)
    buf[*pos] = c;
  (*pos)++;
}

static void append_str(char *buf, size_t cap, size_t *pos, const char *s) {
  for (; *s; s++)
    append_char(buf, cap, pos, *s);
}

static void append_uint(char *buf, size_t cap, size_t *pos,
                        unsigned long long v, unsigned base, int upper,
                        int width, int zero_pad) {
  char digits[32];
  static const char lower[] = "0123456789abcdef";
  static const char upper_d[] = "0123456789ABCDEF";
  const char *tab = upper ? upper_d : lower;
  int n = 0;
  if (v == 0) {
    digits[n++] = '0';
  } else {
    while (v > 0 && n < (int)sizeof(digits)) {
      digits[n++] = tab[v % base];
      v /= base;
    }
  }
  for (int pad = n; pad < width; pad++)
    append_char(buf, cap, pos, zero_pad ? '0' : ' ');
  for (int i = n - 1; i >= 0; i--)
    append_char(buf, cap, pos, digits[i]);
}

int vsnprintf(char *buf, size_t cap, const char *fmt, va_list ap) {
  size_t pos = 0;
  for (const char *p = fmt; *p; p++) {
    if (*p != '%') {
      append_char(buf, cap, &pos, *p);
      continue;
    }
    p++;
    int zero_pad = 0;
    int width = 0;
    while (*p == '0') {
      zero_pad = 1;
      p++;
    }
    while (*p >= '0' && *p <= '9') {
      width = width * 10 + (*p - '0');
      p++;
    }
    int is_size = 0;
    if (p[0] == 'z') {
      is_size = 1;
      p++;
    }
    switch (*p) {
    case 's': {
      const char *s = va_arg(ap, const char *);
      append_str(buf, cap, &pos, s ? s : "(null)");
      break;
    }
    case 'd': {
      long long v = is_size ? (long long)va_arg(ap, size_t) : va_arg(ap, int);
      if (v < 0) {
        append_char(buf, cap, &pos, '-');
        v = -v;
      }
      append_uint(buf, cap, &pos, (unsigned long long)v, 10, 0, width,
                  zero_pad);
      break;
    }
    case 'u': {
      unsigned long long v = is_size ? (unsigned long long)va_arg(ap, size_t)
                                     : va_arg(ap, unsigned int);
      append_uint(buf, cap, &pos, v, 10, 0, width, zero_pad);
      break;
    }
    case 'x': {
      unsigned long long v = is_size ? (unsigned long long)va_arg(ap, size_t)
                                     : va_arg(ap, unsigned int);
      append_uint(buf, cap, &pos, v, 16, 0, width, zero_pad);
      break;
    }
    case 'X': {
      unsigned long long v = is_size ? (unsigned long long)va_arg(ap, size_t)
                                     : va_arg(ap, unsigned int);
      append_uint(buf, cap, &pos, v, 16, 1, width, zero_pad);
      break;
    }
    case 'p': {
      void *v = va_arg(ap, void *);
      append_str(buf, cap, &pos, "0x");
      append_uint(buf, cap, &pos, (unsigned long long)(UINT_PTR)v, 16, 0, 0, 0);
      break;
    }
    case '%':
      append_char(buf, cap, &pos, '%');
      break;
    default:
      append_char(buf, cap, &pos, '%');
      append_char(buf, cap, &pos, *p);
      break;
    }
  }
  if (cap > 0)
    buf[pos < cap ? pos : cap - 1] = '\0';
  return (int)pos;
}

int sprintf(char *buf, const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  int n = vsnprintf(buf, (size_t)-1, fmt, ap);
  va_end(ap);
  return n;
}

int snprintf(char *buf, size_t cap, const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  int n = vsnprintf(buf, cap, fmt, ap);
  va_end(ap);
  return n;
}

int _scprintf(const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  int n = vsnprintf(0, 0, fmt, ap);
  va_end(ap);
  return n;
}

int wp_errno;

struct wp_FILE {
  int unused;
};
static wp_FILE wp_stdout_storage;
static wp_FILE wp_stderr_storage;
wp_FILE *const stdout = &wp_stdout_storage;
wp_FILE *const stderr = &wp_stderr_storage;

int fprintf(wp_FILE *stream, const char *fmt, ...) {
  (void)stream;
  (void)fmt;
  return 0;
}
