#pragma once
#define WALLPIPER_CAPTURE_SOCKET_PATH "/tmp/wallpiper-capture.sock"
#define WALLPIPER_CTL_SOCKET_PATH "/tmp/wallpiper-portal-gnome-ctl.sock"

/* must match layer/siphon/config.h */
#define WP_CAPTURE_SLOT_COUNT 3
#define WP_MAX_CAPTURE_CHANNELS 4

#define MAX_CAPTURE_SLOTS (WP_MAX_CAPTURE_CHANNELS * WP_CAPTURE_SLOT_COUNT)
#define MAX_RECV_FDS 2
