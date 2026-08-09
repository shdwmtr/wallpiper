#if defined(__GNUC__) || defined(__clang__)
#define WP_EXPORT __attribute__((visibility("default")))
#else
#define WP_EXPORT
#endif

#include <dlfcn.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

typedef ssize_t (*write_fn)(int, const void*, size_t);
typedef ssize_t (*read_fn)(int, void*, size_t);

#define REAL_FN(name, type, symbol)                                                                                                                                                \
    static type real_##name(void)                                                                                                                                                  \
    {                                                                                                                                                                              \
        static _Atomic(type) cached = NULL;                                                                                                                                        \
        type fn = atomic_load_explicit(&cached, memory_order_relaxed);                                                                                                             \
        if (fn == NULL) {                                                                                                                                                          \
            fn = (type)dlsym(RTLD_NEXT, symbol);                                                                                                                                   \
            atomic_store_explicit(&cached, fn, memory_order_relaxed);                                                                                                              \
        }                                                                                                                                                                          \
        return fn;                                                                                                                                                                 \
    }

REAL_FN(write, write_fn, "write")
REAL_FN(read, read_fn, "read")

static _Thread_local bool in_hook = false;
static const char selection_socket_name[] = "wallpiper-mitm";

static int socket_fd = -1;
static struct sockaddr_un socket_addr;
static socklen_t socket_addr_len = 0;
static pthread_once_t socket_once = PTHREAD_ONCE_INIT;

static void init_selection_socket(void)
{
    int s = socket(AF_UNIX, SOCK_DGRAM, 0);
    if (s < 0) return;

    memset(&socket_addr, 0, sizeof(socket_addr));
    socket_addr.sun_family = AF_UNIX;
    socket_addr.sun_path[0] = '\0';
    memcpy(socket_addr.sun_path + 1, selection_socket_name, sizeof(selection_socket_name) - 1);
    socket_addr_len = (socklen_t)(offsetof(struct sockaddr_un, sun_path) + 1 + sizeof(selection_socket_name) - 1);
    socket_fd = s;
}

static void send_selection(const char* json, size_t len)
{
    pthread_once(&socket_once, init_selection_socket);
    if (socket_fd < 0) return;
    sendto(socket_fd, json, len, 0, (struct sockaddr*)&socket_addr, socket_addr_len);
}

static bool find_json(const uint8_t* buf, size_t len, size_t* start_out, size_t* end_out)
{
    static const char marker[] = "{\"file\"";
    const size_t marker_len = sizeof(marker) - 1;

    if (len < marker_len) return false;

    size_t start = SIZE_MAX;
    for (size_t i = 0; i + marker_len <= len; i++) {
        if (memcmp(buf + i, marker, marker_len) == 0) {
            start = i;
            break;
        }
    }
    if (start == SIZE_MAX) return false;

    int depth = 0;
    for (size_t i = start; i < len; i++) {
        if (buf[i] == '{') {
            depth++;
        } else if (buf[i] == '}') {
            depth--;
            if (depth == 0) {
                *start_out = start;
                *end_out = i + 1;
                return true;
            }
        }
    }
    return false;
}

static void maybe_log(const uint8_t* buf, size_t len)
{
    size_t start, end;
    if (find_json(buf, len, &start, &end)) {
        send_selection((const char*)buf + start, end - start);
    }
}

WP_EXPORT ssize_t write(int fd, const void* buf, size_t count)
{
    if (!in_hook && buf != NULL && count > 0) {
        in_hook = true;
        maybe_log((const uint8_t*)buf, count);
        in_hook = false;
    }
    return real_write()(fd, buf, count);
}

WP_EXPORT ssize_t read(int fd, void* buf, size_t count)
{
    ssize_t ret = real_read()(fd, buf, count);
    if (!in_hook && ret > 0 && buf != NULL) {
        in_hook = true;
        maybe_log((const uint8_t*)buf, (size_t)ret);
        in_hook = false;
    }
    return ret;
}
