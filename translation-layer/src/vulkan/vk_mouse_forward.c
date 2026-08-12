#include <pthread.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#include "../log.h"
#include "../x11.h"

#define MOUSE_POLL_INTERVAL_US 4000

typedef struct
{
    unsigned long window;
    unsigned long root;
    int width, height;
} MouseTrackArgs;

typedef struct
{
    int x, y;
    int width, height;
} MonitorGeometry;

static int mouse_track_started = 0;

static bool portal_socket_path(char* out, size_t out_len)
{
    const char* path = getenv("WALLPIPER_PORTAL_CTL_SOCKET");
    if (!path || path[0] == '\0') {
        return false;
    }
    int n = snprintf(out, out_len, "%s", path);
    return n > 0 && (size_t)n < out_len;
}

static bool portal_cursor_pos(int* x, int* y)
{
    char path[256];
    if (!portal_socket_path(path, sizeof(path))) {
        return false;
    }
    size_t path_len = strlen(path);
    if (path_len >= sizeof(((struct sockaddr_un*)0)->sun_path)) {
        return false;
    }

    int sock = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sock < 0) {
        return false;
    }

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    memcpy(addr.sun_path, path, path_len);

    if (connect(sock, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        close(sock);
        return false;
    }

    static const char cmd[] = "CURSOR_POS\n";
    if (write(sock, cmd, sizeof(cmd) - 1) != (ssize_t)(sizeof(cmd) - 1)) {
        close(sock);
        return false;
    }

    char buf[64];
    size_t total = 0;
    while (total < sizeof(buf) - 1) {
        ssize_t n = read(sock, buf + total, sizeof(buf) - 1 - total);
        if (n <= 0) {
            break;
        }
        total += (size_t)n;
    }
    close(sock);
    buf[total] = '\0';

    return sscanf(buf, "CURSOR_POS %d %d", x, y) == 2;
}

static bool env_int(const char* name, int* out)
{
    const char* value = getenv(name);
    if (!value || value[0] == '\0') {
        return false;
    }
    char* end = NULL;
    long parsed = strtol(value, &end, 10);
    if (end == value) {
        return false;
    }
    *out = (int)parsed;
    return true;
}

static MonitorGeometry monitor_geometry_from_env(int fallback_width, int fallback_height)
{
    MonitorGeometry geometry = { 0, 0, fallback_width, fallback_height };
    bool have_x = env_int("WALLPIPER_MONITOR_X", &geometry.x);
    bool have_y = env_int("WALLPIPER_MONITOR_Y", &geometry.y);
    bool have_w = env_int("WALLPIPER_MONITOR_LOGICAL_WIDTH", &geometry.width);
    bool have_h = env_int("WALLPIPER_MONITOR_LOGICAL_HEIGHT", &geometry.height);

    if (have_x && have_y && have_w && have_h && geometry.width > 0 && geometry.height > 0) {
        wp_log("mouse track: monitor geometry from env: %dx%d at %d,%d", geometry.width, geometry.height, geometry.x, geometry.y);
    } else {
        geometry.x = 0;
        geometry.y = 0;
        geometry.width = fallback_width;
        geometry.height = fallback_height;
        wp_log("mouse track: WALLPIPER_MONITOR_* env vars missing/invalid, assuming origin 0,0 and 1:1 scale");
    }
    return geometry;
}

static void send_synthetic_motion(pfn_xsendevent send_event, pfn_xflush flush, void* display, unsigned long window, unsigned long root, int wx, int wy, int gx, int gy)
{
    XEventCompat ev;
    memset(&ev, 0, sizeof(ev));
    ev.xmotion.type = X_MOTION_NOTIFY;
    ev.xmotion.send_event = 1;
    ev.xmotion.display = display;
    ev.xmotion.window = window;
    ev.xmotion.root = root;
    ev.xmotion.x = wx;
    ev.xmotion.y = wy;
    ev.xmotion.x_root = gx;
    ev.xmotion.y_root = gy;
    ev.xmotion.same_screen = 1;
    send_event(display, window, 1, X_POINTER_MOTION_MASK, &ev);
    flush(display);
}

static bool map_global_to_window(const MonitorGeometry* monitor, int width, int height, int* wx, int* wy, int* out_gx, int* out_gy)
{
    int gx, gy;
    if (!portal_cursor_pos(&gx, &gy)) {
        return false;
    }
    *out_gx = gx;
    *out_gy = gy;

    int lx = gx - monitor->x;
    int ly = gy - monitor->y;
    if (lx < 0) {
        lx = 0;
    }
    if (lx >= monitor->width) {
        lx = monitor->width - 1;
    }
    if (ly < 0) {
        ly = 0;
    }
    if (ly >= monitor->height) {
        ly = monitor->height - 1;
    }

    int rx = (int)((double)lx * width / monitor->width);
    int ry = (int)((double)ly * height / monitor->height);
    if (rx < 0) {
        rx = 0;
    }
    if (rx >= width) {
        rx = width - 1;
    }
    if (ry < 0) {
        ry = 0;
    }
    if (ry >= height) {
        ry = height - 1;
    }
    *wx = rx;
    *wy = ry;
    return true;
}

static void* mouse_track_thread(void* raw_args)
{
    MouseTrackArgs* args = (MouseTrackArgs*)raw_args;
    unsigned long window = args->window;
    unsigned long root = args->root;
    int width = args->width;
    int height = args->height;
    free(args);

    pfn_xopendisplay open_display = (pfn_xopendisplay)interpose_resolve("XOpenDisplay");
    pfn_xsendevent send_event = (pfn_xsendevent)interpose_resolve("XSendEvent");
    pfn_xflush flush = (pfn_xflush)interpose_resolve("XFlush");
    if (!open_display || !send_event || !flush) {
        wp_log("mouse track: failed to resolve XOpenDisplay/XSendEvent/XFlush");
        return NULL;
    }

    void* display = open_display(NULL);
    if (!display) {
        wp_log("mouse track: XOpenDisplay(NULL) failed");
        return NULL;
    }

    MonitorGeometry monitor = monitor_geometry_from_env(width, height);

    wp_log("mouse track: polling cursor position on fixed interval");

    int wx, wy, gx, gy;
    for (;;) {
        if (map_global_to_window(&monitor, width, height, &wx, &wy, &gx, &gy)) {
            send_synthetic_motion(send_event, flush, display, window, root, wx, wy, gx, gy);
        }
        usleep(MOUSE_POLL_INTERVAL_US);
    }

    return NULL;
}

void mouse_track_start(void* display, unsigned long window, int width, int height)
{
    int expected = 0;
    if (!__atomic_compare_exchange_n(&mouse_track_started, &expected, 1, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST)) {
        return;
    }

    pfn_xgetgeometry get_geometry = (pfn_xgetgeometry)interpose_resolve("XGetGeometry");
    unsigned long root = 0;
    int gx = 0, gy = 0;
    unsigned int gwidth = 0, gheight = 0, border = 0, depth = 0;
    if (!get_geometry || !get_geometry(display, window, &root, &gx, &gy, &gwidth, &gheight, &border, &depth)) {
        wp_log("mouse track: XGetGeometry (for root) failed, skipping");
        return;
    }

    MouseTrackArgs* args = malloc(sizeof(MouseTrackArgs));
    if (!args) {
        return;
    }
    args->window = window;
    args->root = root;
    args->width = width;
    args->height = height;

    pthread_t thread;
    if (pthread_create(&thread, NULL, mouse_track_thread, args) == 0) {
        pthread_detach(thread);
        wp_log("mouse track: thread spawned");
    } else {
        wp_log("mouse track: pthread_create failed");
        free(args);
    }
}
