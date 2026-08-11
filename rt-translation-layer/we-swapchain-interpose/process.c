#include "process.h"

#include <stddef.h>
#include <stdio.h>
#include <string.h>

static const char* const target_process_names[] = { "wallpaper64.exe" };
static const char* const target_cmdline_markers[] = { "webwallpaper64.exe" };

static bool read_comm(char* out, size_t out_len)
{
    FILE* f = fopen("/proc/self/comm", "r");
    if (!f) {
        return false;
    }
    size_t n = fread(out, 1, out_len - 1, f);
    fclose(f);
    while (n > 0 && (out[n - 1] == '\n' || out[n - 1] == '\r')) {
        n--;
    }
    out[n] = '\0';
    return true;
}

bool interpose_is_target_process(void)
{
    static int cached = -1;
    int c = __atomic_load_n(&cached, __ATOMIC_RELAXED);
    if (c != -1) {
        return c != 0;
    }

    bool result = false;

    char comm[256];
    if (read_comm(comm, sizeof(comm))) {
        for (size_t i = 0; i < sizeof(target_process_names) / sizeof(target_process_names[0]); i++) {
            if (strcmp(comm, target_process_names[i]) == 0) {
                result = true;
                break;
            }
        }
    }

    if (!result) {
        FILE* cf = fopen("/proc/self/cmdline", "r");
        if (cf) {
            char buf[4096];
            size_t n = fread(buf, 1, sizeof(buf), cf);
            fclose(cf);
            for (size_t i = 0; i < sizeof(target_cmdline_markers) / sizeof(target_cmdline_markers[0]); i++) {
                size_t marker_len = strlen(target_cmdline_markers[i]);
                if (marker_len > 0 && n >= marker_len && memmem(buf, n, target_cmdline_markers[i], marker_len) != NULL) {
                    result = true;
                    break;
                }
            }
        }
    }

    __atomic_store_n(&cached, result ? 1 : 0, __ATOMIC_RELAXED);
    return result;
}

bool interpose_is_wine_shell_process(void)
{
    char comm[256];
    return read_comm(comm, sizeof(comm)) && strcmp(comm, "explorer.exe") == 0;
}
