#if defined(__GNUC__) || defined(__clang__)
#define WP_EXPORT __attribute__((visibility("default")))
#else
#define WP_EXPORT
#endif

#include <dlfcn.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static const char wallpiper_window_tag[] = "wallpiper-";
static uint64_t map_suppress_count = 0;

typedef struct {
  int type;
  unsigned long serial;
  int send_event;
  void *display;
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

typedef struct {
  int type;
  unsigned long serial;
  int send_event;
  void *display;
  unsigned long window;
  unsigned long root;
  unsigned long subwindow;
  unsigned long time;
  int x, y;
  int x_root, y_root;
  unsigned int state;
  unsigned int keycode;
  int same_screen;
} XKeyEventCompat;

typedef union {
  int type;
  XMotionEventCompat xmotion;
  XKeyEventCompat xkey;
  long pad[24];
} XEventCompat;

typedef struct {
  int max_keypermod;
  unsigned char *modifiermap;
} XModifierKeymapCompat;

typedef struct {
  int width, height;
  int xoffset;
  int format;
  char *data;
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

typedef struct {
  int x, y;
  int width, height;
  int border_width;
  int depth;
  void *visual;
  unsigned long root;
  int win_class;
  int bit_gravity;
  int win_gravity;
  int backing_store;
  unsigned long backing_planes;
  unsigned long backing_pixel;
  int save_under;
  unsigned long colormap;
  int map_installed;
  int map_state;
  long all_event_masks;
  long your_event_mask;
  long do_not_propagate_mask;
  int override_redirect;
  void *screen;
} XWindowAttributesCompat;

#define X_MOTION_NOTIFY 6
#define X_POINTER_MOTION_MASK (1L << 6)

#define X_KEY_PRESS 2
#define X_KEY_RELEASE 3
#define X_KEY_PRESS_MASK (1L << 0)
#define X_KEY_RELEASE_MASK (1L << 1)

typedef void *(*pfn_xopendisplay)(const char *);
typedef int (*pfn_xsendevent)(void *, unsigned long, int, long, XEventCompat *);
typedef int (*pfn_xflush)(void *);
typedef int (*pfn_xfetchname)(void *, unsigned long, char **);
typedef int (*pfn_xfree)(void *);
typedef int (*pfn_xgetgeometry)(void *, unsigned long, unsigned long *, int *,
                                int *, unsigned int *, unsigned int *,
                                unsigned int *, unsigned int *);
typedef int (*pfn_xmapwindow)(void *, unsigned long);
typedef int (*pfn_xshmputimage)(void *, unsigned long, void *, XImageCompat *,
                                int, int, int, int, unsigned int, unsigned int,
                                int);
typedef int (*pfn_xgetwindowattributes)(void *, unsigned long,
                                        XWindowAttributesCompat *);
typedef unsigned long (*pfn_xstringtokeysym)(const char *);
typedef unsigned int (*pfn_xkeysymtokeycode)(void *, unsigned long);
typedef XModifierKeymapCompat *(*pfn_xgetmodifiermapping)(void *);
typedef int (*pfn_xfreemodifiermap)(XModifierKeymapCompat *);

inline void *interpose_resolve(const char *symbol_name) {
  return dlsym(RTLD_NEXT, symbol_name);
}

inline char *x11_window_title(void *display, unsigned long window) {
  pfn_xfetchname fetch = (pfn_xfetchname)interpose_resolve("XFetchName");
  if (!fetch) {
    return NULL;
  }
  pfn_xfree free_fn = (pfn_xfree)interpose_resolve("XFree");

  char *name_ptr = NULL;
  int status = fetch(display, window, &name_ptr);
  if (status == 0 || name_ptr == NULL) {
    return NULL;
  }
  char *title = strdup(name_ptr);
  if (free_fn) {
    free_fn(name_ptr);
  }
  return title;
}

inline bool x11_window_attributes(void *display, unsigned long window,
                                  XWindowAttributesCompat *out) {
  pfn_xgetwindowattributes get_attrs =
      (pfn_xgetwindowattributes)interpose_resolve("XGetWindowAttributes");
  if (!get_attrs) {
    return false;
  }
  return get_attrs(display, window, out) != 0;
}

WP_EXPORT int XMapWindow(void *display, unsigned long window) {
  pfn_xmapwindow real = (pfn_xmapwindow)interpose_resolve("XMapWindow");
  if (!real) {
    return 1;
  }

  char *title = x11_window_title(display, window);
  bool is_ours =
      title != NULL && strncmp(title, wallpiper_window_tag,
                               sizeof(wallpiper_window_tag) - 1) == 0;
  bool is_empty_titled = title != NULL && title[0] == '\0';

  free(title);

  bool is_border = false;
  if (!is_ours && is_empty_titled) {
    XWindowAttributesCompat attrs;
    is_border = x11_window_attributes(display, window, &attrs) &&
                attrs.override_redirect;
  }

  if (!is_ours && !is_border && !is_empty_titled) {
    return real(display, window);
  }

  __atomic_fetch_add(&map_suppress_count, 1, __ATOMIC_RELAXED);
  return 1;
}
