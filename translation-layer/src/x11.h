#pragma once
#include <dlfcn.h>
#include <string.h>
#include <stdbool.h>

typedef struct
{
    int type;
    unsigned long serial;
    int send_event;
    void* display;
    unsigned long window;
    unsigned long root;
    unsigned long subwindow;
    unsigned long time;
    int x, y;
    int x_root, y_root;
    unsigned int state;
    char is_hint;
    int same_screen;
} XMotionEventCompat;

typedef union
{
    int type;
    XMotionEventCompat xmotion;
    long pad[24];
} XEventCompat;

typedef struct
{
    int width, height;
    int xoffset;
    int format;
    char* data;
    int byte_order;
    int bitmap_unit;
    int bitmap_bit_order;
    int bitmap_pad;
    int depth;
    int bytes_per_line;
    int bits_per_pixel;
    unsigned long red_mask;
    unsigned long green_mask;
    unsigned long blue_mask;
} XImageCompat;

#define X_MOTION_NOTIFY 6
#define X_POINTER_MOTION_MASK (1L << 6)

typedef void* (*pfn_xopendisplay)(const char*);
typedef int (*pfn_xsendevent)(void*, unsigned long, int, long, XEventCompat*);
typedef int (*pfn_xflush)(void*);
typedef int (*pfn_xfetchname)(void*, unsigned long, char**);
typedef int (*pfn_xfree)(void*);
typedef int (*pfn_xgetgeometry)(void*, unsigned long, unsigned long*, int*, int*, unsigned int*, unsigned int*, unsigned int*, unsigned int*);
typedef int (*pfn_xmapwindow)(void*, unsigned long);
typedef int (*pfn_xshmputimage)(void*, unsigned long, void*, XImageCompat*, int, int, int, int, unsigned int, unsigned int, int);

inline void* interpose_resolve(const char* symbol_name)
{
    return dlsym(RTLD_NEXT, symbol_name);
}

inline char* x11_window_title(void* display, unsigned long window)
{
    pfn_xfetchname fetch = (pfn_xfetchname)interpose_resolve("XFetchName");
    if (!fetch) {
        return NULL;
    }
    pfn_xfree free_fn = (pfn_xfree)interpose_resolve("XFree");

    char* name_ptr = NULL;
    int status = fetch(display, window, &name_ptr);
    if (status == 0 || name_ptr == NULL) {
        return NULL;
    }
    char* title = strdup(name_ptr);
    if (free_fn) {
        free_fn(name_ptr);
    }
    return title;
}

inline bool x11_window_size(void* display, unsigned long window, unsigned int* width, unsigned int* height)
{
    pfn_xgetgeometry get_geometry = (pfn_xgetgeometry)interpose_resolve("XGetGeometry");
    if (!get_geometry) {
        return false;
    }

    unsigned long root = 0;
    int x = 0, y = 0;
    unsigned int border_width = 0, depth = 0;

    int status = get_geometry(display, window, &root, &x, &y, width, height, &border_width, &depth);
    return status != 0;
}
