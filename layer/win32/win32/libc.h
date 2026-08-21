#ifndef WP_WIN32_LIBC_H
#define WP_WIN32_LIBC_H

#include "types.h"

void *malloc(size_t size);
void *calloc(size_t count, size_t size);
void *realloc(void *ptr, size_t size);
void free(void *ptr);

void *memcpy(void *dst, const void *src, size_t n);
void *memmove(void *dst, const void *src, size_t n);
void *memset(void *dst, int c, size_t n);
int memcmp(const void *a, const void *b, size_t n);

size_t strlen(const char *s);
int strcmp(const char *a, const char *b);
int strncmp(const char *a, const char *b, size_t n);
char *strcpy(char *dst, const char *src);
char *strchr(const char *s, int c);
char *strstr(const char *haystack, const char *needle);
int strcasecmp(const char *a, const char *b);
#define stricmp strcasecmp

int sprintf(char *buf, const char *fmt, ...);
int snprintf(char *buf, size_t cap, const char *fmt, ...);
int vsnprintf(char *buf, size_t cap, const char *fmt, va_list ap);
int _scprintf(const char *fmt, ...);

typedef struct wp_FILE wp_FILE;
#define FILE wp_FILE
extern wp_FILE *const stdout;
extern wp_FILE *const stderr;
int fprintf(wp_FILE *stream, const char *fmt, ...);

extern int wp_errno;
#define errno wp_errno

#endif
