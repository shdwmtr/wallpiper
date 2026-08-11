#include "capture_listener.h"
#include "egl_import.h"
#include "error.h"

#include <glib-unix.h>
#include <errno.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

void wallpiper_capture_listener_detach(WallpiperPortalState* state)
{
    for (int i = 0; i < MAX_CAPTURE_SLOTS; i++) {
        WallpiperCaptureSlot* slot = &state->slots[i];
        if (slot->texture) {
            g_object_unref(slot->texture);
            slot->texture = NULL;
        }
        slot->used = FALSE;
    }
    clutter_actor_set_content(state->display_actor, NULL);
    g_message("wallpiper-gnome: detached, cleared all slots");
}

static void display_slot(WallpiperPortalState* state, WallpiperCaptureSlot* slot)
{
    ClutterContent* content = clutter_texture_content_new_from_texture(slot->texture, NULL);
    clutter_actor_set_size(state->display_actor, slot->width, slot->height);
    clutter_actor_set_content(state->display_actor, content);
}

static gboolean recv_capture_message(int sock_fd, char* header_buf, gsize header_buf_size, gssize* out_header_len, int* out_fds, int* out_n_fds)
{
    struct iovec iov = {
        .iov_base = header_buf,
        .iov_len = header_buf_size,
    };
    char cmsg_buf[64];
    struct msghdr msg;
    memset(&msg, 0, sizeof(msg));
    msg.msg_iov = &iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf;
    msg.msg_controllen = sizeof(cmsg_buf);

    ssize_t n = recvmsg(sock_fd, &msg, 0);
    if (n < 0) return FALSE;

    *out_header_len = n;
    *out_n_fds = 0;

    struct cmsghdr* cmsg = CMSG_FIRSTHDR(&msg);
    if (cmsg && cmsg->cmsg_level == SOL_SOCKET && cmsg->cmsg_type == SCM_RIGHTS) {
        size_t data_len = cmsg->cmsg_len - CMSG_LEN(0);
        int count = (int)(data_len / sizeof(int));
        if (count > MAX_RECV_FDS) count = MAX_RECV_FDS;
        int* fds = (int*)CMSG_DATA(cmsg);
        for (int i = 0; i < count; i++)
            out_fds[i] = fds[i];
        *out_n_fds = count;
    }

    return TRUE;
}

static void handle_buf_message(WallpiperPortalState* state, char** parts, guint n_parts, int* fds, int n_fds)
{
    if (n_parts < 7) {
        g_warning("wallpiper-gnome: malformed BUF message (%u fields)", n_parts);
        for (int i = 0; i < n_fds; i++)
            close(fds[i]);
        return;
    }

    guint32 slot_index = (guint32)g_ascii_strtoull(parts[1], NULL, 10);
    guint32 width = (guint32)g_ascii_strtoull(parts[2], NULL, 10);
    guint32 height = (guint32)g_ascii_strtoull(parts[3], NULL, 10);
    guint32 stride = (guint32)g_ascii_strtoull(parts[5], NULL, 10);
    guint64 modifier = g_ascii_strtoull(parts[6], NULL, 10);

    int dmabuf_fd = n_fds > 0 ? fds[0] : -1;
    int sync_fd = n_fds > 1 ? fds[1] : -1;
    wallpiper_egl_wait_sync_fd(state->egl_display, sync_fd);

    if (dmabuf_fd < 0 || slot_index >= MAX_CAPTURE_SLOTS) {
        g_warning("wallpiper-gnome: bad BUF message (slot=%u fd=%d)", slot_index, dmabuf_fd);
        if (dmabuf_fd >= 0) close(dmabuf_fd);
        return;
    }

    g_message("wallpiper-gnome: BUF slot=%u %ux%u stride=%u modifier=0x%llx fd=%d", slot_index, width, height, stride, (unsigned long long)modifier, dmabuf_fd);

    GError* local_error = NULL;
    CoglTexture* texture = wallpiper_egl_import_dmabuf(state->cogl_context, state->egl_display, dmabuf_fd, width, height, stride, 0, modifier, &local_error);
    close(dmabuf_fd);

    if (!texture) {
        g_warning("wallpiper-gnome: failed to import BUF slot %u: %s", slot_index, local_error ? local_error->message : "unknown error");
        g_clear_error(&local_error);
        return;
    }

    WallpiperCaptureSlot* slot = &state->slots[slot_index];
    if (slot->texture) g_object_unref(slot->texture);
    slot->used = TRUE;
    slot->width = width;
    slot->height = height;
    slot->texture = texture;

    display_slot(state, slot);

    g_message("wallpiper-gnome: displaying slot %u", slot_index);
}

static void handle_frame_message(WallpiperPortalState* state, char** parts, guint n_parts, int* fds, int n_fds)
{
    int sync_fd = n_fds > 0 ? fds[0] : -1;
    wallpiper_egl_wait_sync_fd(state->egl_display, sync_fd);

    if (n_parts < 2) return;

    guint32 slot_index = (guint32)g_ascii_strtoull(parts[1], NULL, 10);
    if (slot_index >= MAX_CAPTURE_SLOTS || !state->slots[slot_index].used) return;

    display_slot(state, &state->slots[slot_index]);
    g_message("wallpiper-gnome: FRAME -> displaying slot %u", slot_index);
}

static gboolean on_capture_socket_readable(gint fd, GIOCondition condition, gpointer user_data)
{
    WallpiperPortalState* state = user_data;

    if (condition & (G_IO_ERR | G_IO_HUP)) {
        g_warning("wallpiper-gnome: capture socket error/hangup");
        return G_SOURCE_REMOVE;
    }

    char header_buf[256];
    gssize header_len = 0;
    int fds[MAX_RECV_FDS];
    int n_fds = 0;

    if (!recv_capture_message(fd, header_buf, sizeof(header_buf) - 1, &header_len, fds, &n_fds)) {
        g_warning("wallpiper-gnome: recvmsg failed: %s", g_strerror(errno));
        return G_SOURCE_CONTINUE;
    }

    header_buf[header_len] = '\0';

    char** parts = g_strsplit(g_strstrip(header_buf), " ", -1);
    guint n_parts = g_strv_length(parts);

    if (n_parts >= 1 && g_strcmp0(parts[0], "BUF") == 0)
        handle_buf_message(state, parts, n_parts, fds, n_fds);
    else if (n_parts >= 1 && g_strcmp0(parts[0], "FRAME") == 0)
        handle_frame_message(state, parts, n_parts, fds, n_fds);
    else {
        g_message("wallpiper-gnome: unrecognized capture message: %s", header_buf);
        for (int i = 0; i < n_fds; i++)
            close(fds[i]);
    }

    g_strfreev(parts);
    return G_SOURCE_CONTINUE;
}

gboolean wallpiper_capture_listener_start(WallpiperPortalState* state, GError** error)
{
    int capture_fd = socket(AF_UNIX, SOCK_DGRAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0);
    if (capture_fd < 0) {
        g_set_error(error, WALLPIPER_ERROR, 0, "capture socket() failed: %s", g_strerror(errno));
        return FALSE;
    }

    unlink(WALLPIPER_CAPTURE_SOCKET_PATH);

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    g_strlcpy(addr.sun_path, WALLPIPER_CAPTURE_SOCKET_PATH, sizeof(addr.sun_path));
    if (bind(capture_fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        g_set_error(error, WALLPIPER_ERROR, 0, "bind(%s) failed: %s", WALLPIPER_CAPTURE_SOCKET_PATH, g_strerror(errno));
        close(capture_fd);
        return FALSE;
    }

    state->capture_socket_fd = capture_fd;
    state->capture_source_id = g_unix_fd_add(capture_fd, G_IO_IN, on_capture_socket_readable, state);

    return TRUE;
}

void wallpiper_capture_listener_stop(WallpiperPortalState* state)
{
    g_source_remove(state->capture_source_id);
    close(state->capture_socket_fd);
    unlink(WALLPIPER_CAPTURE_SOCKET_PATH);

    for (int i = 0; i < MAX_CAPTURE_SLOTS; i++) {
        WallpiperCaptureSlot* slot = &state->slots[i];
        if (slot->texture) {
            g_object_unref(slot->texture);
            slot->texture = NULL;
        }
        slot->used = FALSE;
    }
}
