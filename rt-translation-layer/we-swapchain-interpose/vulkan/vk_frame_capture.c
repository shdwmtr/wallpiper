#include <errno.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#include "vk_api.h"
#include "../log.h"
#include "../process.h"

#define CAPTURE_SOCKET_PATH "/tmp/wallpiper-capture.sock"
#define MIN_CAPTURE_WIDTH 800
#define MIN_CAPTURE_HEIGHT 600

static int send_fd_raw(int sock_fd, const char* header, size_t header_len, int fd)
{
    struct iovec iov = {
        .iov_base = (void*)header,
        .iov_len = header_len,
    };

    union
    {
        char buf[64];
        struct cmsghdr align;
    } cmsg_buf;

    size_t cmsg_space = CMSG_SPACE(sizeof(int));
    if (cmsg_space > sizeof(cmsg_buf.buf)) {
        return -1;
    }
    memset(&cmsg_buf, 0, sizeof(cmsg_buf));

    struct msghdr msg;
    memset(&msg, 0, sizeof(msg));
    msg.msg_iov = &iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.buf;
    msg.msg_controllen = cmsg_space;

    struct cmsghdr* cmsg = CMSG_FIRSTHDR(&msg);
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;
    cmsg->cmsg_len = CMSG_LEN(sizeof(int));
    memcpy(CMSG_DATA(cmsg), &fd, sizeof(int));

    ssize_t ret = sendmsg(sock_fd, &msg, 0);
    if (ret < 0) {
        return -1;
    }
    return 0;
}

static void send_frame(int width, int height, int stride, const char* data)
{
    size_t len = (size_t)(stride > 0 ? stride : 0) * (size_t)(height > 0 ? height : 0);
    if (data == NULL || len == 0) {
        return;
    }

    int memfd = memfd_create("we-shm-capture-frame", 0);
    if (memfd < 0) {
        wp_log("memfd_create failed: %s", strerror(errno));
        return;
    }

    ssize_t written = write(memfd, data, len);
    if (written < 0 || (size_t)written != len) {
        wp_log("memfd write failed or short: %zd vs %zu", written, len);
        close(memfd);
        return;
    }

    int sock = socket(AF_UNIX, SOCK_DGRAM, 0);
    if (sock < 0) {
        close(memfd);
        return;
    }
    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, CAPTURE_SOCKET_PATH, sizeof(addr.sun_path) - 1);

    if (connect(sock, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        wp_log("no capture receiver listening yet: %s", strerror(errno));
        close(sock);
        close(memfd);
        return;
    }

    char header[128];
    int header_len = snprintf(header, sizeof(header), "SHM %d %d %d\n", width, height, stride);

    int result = send_fd_raw(sock, header, (size_t)header_len, memfd);
    close(memfd);
    close(sock);

    if (result != 0) {
        wp_log("send_frame failed: %s", strerror(errno));
    }
}

void capture_on_put_image([[maybe_unused]] unsigned long drawable, const XImageCompat* image, [[maybe_unused]] int src_x, [[maybe_unused]] int src_y, [[maybe_unused]] int dst_x,
                          [[maybe_unused]] int dst_y, [[maybe_unused]] unsigned int src_width, [[maybe_unused]] unsigned int src_height)
{
    if (!interpose_is_target_process() || image == NULL) {
        return;
    }
    if (image->width < MIN_CAPTURE_WIDTH || image->height < MIN_CAPTURE_HEIGHT) {
        return;
    }

    send_frame(image->width, image->height, image->bytes_per_line, image->data);
}
