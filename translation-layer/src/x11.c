#if defined(__GNUC__) || defined(__clang__)
#define WP_EXPORT __attribute__((visibility("default")))
#else
#define WP_EXPORT
#endif

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "vulkan/vk_api.h"
#include "log.h"
#include "process.h"
#include "x11.h"

static const char wallpiper_window_tag[] = "wallpiper-";
static uint64_t map_suppress_count = 0;

WP_EXPORT int XMapWindow(void* display, unsigned long window)
{
    pfn_xmapwindow real = (pfn_xmapwindow)interpose_resolve("XMapWindow");
    if (!real) {
        return 1;
    }

    char* title = x11_window_title(display, window);
    unsigned int width = 0, height = 0;
    bool has_size = x11_window_size(display, window, &width, &height);
    bool tray_shaped = interpose_is_wine_shell_process();

    char size_buf[64];
    if (has_size) {
        snprintf(size_buf, sizeof(size_buf), "Some((%u, %u))", width, height);
    } else {
        snprintf(size_buf, sizeof(size_buf), "None");
    }
    wp_log("XMapWindow called, window=0x%lx, title=%s%s%s, size=%s, tray_shaped=%s", window, title ? "Some(\"" : "None", title ? title : "", title ? "\")" : "", size_buf,
           tray_shaped ? "true" : "false");

    bool is_ours = title != NULL && strncmp(title, wallpiper_window_tag, sizeof(wallpiper_window_tag) - 1) == 0;
    free(title);

    if (!(is_ours || tray_shaped)) {
        return real(display, window);
    }

    uint64_t n = __atomic_fetch_add(&map_suppress_count, 1, __ATOMIC_RELAXED) + 1;
    wp_log("XMapWindow suppressed, window=0x%lx, call #%llu", window, (unsigned long long)n);

    if (is_ours && has_size) {
        mouse_track_start(display, window, (int)width, (int)height);
    }

    return 1;
}
