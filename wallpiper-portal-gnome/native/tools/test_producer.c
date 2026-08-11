#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#include <gbm.h>
#include <drm_fourcc.h>

#define CAPTURE_SOCKET_PATH "/tmp/wallpiper-capture.sock"
#define TEST_WIDTH 384
#define TEST_HEIGHT 384

static struct gbm_device* open_render_node(int* out_fd)
{
    static const char* candidates[] = {
        "/dev/dri/renderD128",
        "/dev/dri/renderD129",
        "/dev/dri/renderD130",
        NULL,
    };

    for (int i = 0; candidates[i]; i++) {
        int fd = open(candidates[i], O_RDWR | O_CLOEXEC);
        if (fd < 0) continue;

        struct gbm_device* dev = gbm_create_device(fd);
        if (dev) {
            *out_fd = fd;
            return dev;
        }

        close(fd);
    }

    return NULL;
}

static void fill_pattern(struct gbm_bo* bo)
{
    uint32_t stride = 0;
    void* map_data = NULL;
    uint8_t* pixels = gbm_bo_map(bo, 0, 0, TEST_WIDTH, TEST_HEIGHT, GBM_BO_TRANSFER_WRITE, &stride, &map_data);
    if (!pixels) return;

    for (int y = 0; y < TEST_HEIGHT; y++) {
        uint32_t* row = (uint32_t*)(pixels + y * stride);
        for (int x = 0; x < TEST_WIDTH; x++) {
            int stripe = ((x + y) / 6) % 2 == 0;
            uint8_t r = stripe ? 40 : 220;
            uint8_t g = stripe ? 180 : 40;
            uint8_t b = 90;
            row[x] = (0xffu << 24) | ((uint32_t)r << 16) | ((uint32_t)g << 8) | b;
        }
    }

    gbm_bo_unmap(bo, map_data);
}

static int send_with_fd(int sock_fd, const char* header, int fd)
{
    struct iovec iov = {
        .iov_base = (void*)header,
        .iov_len = strlen(header),
    };

    char cmsg_buf[CMSG_SPACE(sizeof(int))];
    memset(cmsg_buf, 0, sizeof(cmsg_buf));

    struct msghdr msg;
    memset(&msg, 0, sizeof(msg));
    msg.msg_iov = &iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf;
    msg.msg_controllen = sizeof(cmsg_buf);

    struct cmsghdr* cmsg = CMSG_FIRSTHDR(&msg);
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;
    cmsg->cmsg_len = CMSG_LEN(sizeof(int));
    memcpy(CMSG_DATA(cmsg), &fd, sizeof(int));

    return sendmsg(sock_fd, &msg, 0) >= 0 ? 0 : -1;
}

int main(int argc, char** argv)
{
    uint32_t slot = argc > 1 ? (uint32_t)atoi(argv[1]) : 0;

    int render_fd = -1;
    struct gbm_device* gbm_dev = open_render_node(&render_fd);
    if (!gbm_dev) {
        fprintf(stderr, "test-producer: could not open a DRM render node\n");
        return 1;
    }

    struct gbm_bo* bo = gbm_bo_create(gbm_dev, TEST_WIDTH, TEST_HEIGHT, GBM_FORMAT_XRGB8888, GBM_BO_USE_LINEAR | GBM_BO_USE_RENDERING);
    if (!bo) bo = gbm_bo_create(gbm_dev, TEST_WIDTH, TEST_HEIGHT, GBM_FORMAT_XRGB8888, GBM_BO_USE_RENDERING);
    if (!bo) bo = gbm_bo_create(gbm_dev, TEST_WIDTH, TEST_HEIGHT, GBM_FORMAT_XRGB8888, GBM_BO_USE_SCANOUT | GBM_BO_USE_RENDERING);
    if (!bo) {
        fprintf(stderr, "test-producer: gbm_bo_create failed: %s\n", strerror(errno));
        gbm_device_destroy(gbm_dev);
        close(render_fd);
        return 1;
    }

    fill_pattern(bo);

    int dmabuf_fd = gbm_bo_get_fd(bo);
    uint32_t stride = gbm_bo_get_stride(bo);
    uint64_t modifier = gbm_bo_get_modifier(bo);

    printf("test-producer: dmabuf fd=%d %ux%u stride=%u modifier=0x%llx slot=%u\n", dmabuf_fd, TEST_WIDTH, TEST_HEIGHT, stride, (unsigned long long)modifier, slot);

    int sock_fd = socket(AF_UNIX, SOCK_DGRAM | SOCK_CLOEXEC, 0);
    if (sock_fd < 0) {
        fprintf(stderr, "test-producer: socket() failed: %s\n", strerror(errno));
        close(dmabuf_fd);
        gbm_bo_destroy(bo);
        gbm_device_destroy(gbm_dev);
        close(render_fd);
        return 1;
    }

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, CAPTURE_SOCKET_PATH, sizeof(addr.sun_path) - 1);

    if (connect(sock_fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) {
        fprintf(stderr, "test-producer: connect(%s) failed: %s (is the portal listening?)\n", CAPTURE_SOCKET_PATH, strerror(errno));
        close(sock_fd);
        close(dmabuf_fd);
        gbm_bo_destroy(bo);
        gbm_device_destroy(gbm_dev);
        close(render_fd);
        return 1;
    }

    char buf_header[256];
    snprintf(buf_header, sizeof(buf_header), "BUF %u %u %u 0 %u %llu", slot, (unsigned)TEST_WIDTH, (unsigned)TEST_HEIGHT, stride, (unsigned long long)modifier);

    if (send_with_fd(sock_fd, buf_header, dmabuf_fd) != 0) {
        fprintf(stderr, "test-producer: sendmsg(BUF) failed: %s\n", strerror(errno));
        close(sock_fd);
        close(dmabuf_fd);
        gbm_bo_destroy(bo);
        gbm_device_destroy(gbm_dev);
        close(render_fd);
        return 1;
    }
    printf("test-producer: sent %s\n", buf_header);

    close(dmabuf_fd);
    gbm_bo_destroy(bo);
    gbm_device_destroy(gbm_dev);
    close(render_fd);

    char frame_header[64];
    snprintf(frame_header, sizeof(frame_header), "FRAME %u", slot);
    if (send(sock_fd, frame_header, strlen(frame_header), 0) < 0) {
        fprintf(stderr, "test-producer: send(FRAME) failed: %s\n", strerror(errno));
        close(sock_fd);
        return 1;
    }
    printf("test-producer: sent %s\n", frame_header);

    close(sock_fd);
    return 0;
}
