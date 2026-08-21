CC ?= cc
CFLAGS ?= -std=gnu99 -O2 -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE -Iinclude -I../vendor/cjson

TARGET_DIR := ../target/release
OBJ_DIR := ../target/objects/wallpiper-protocol
LIB := $(TARGET_DIR)/libwallpiper-protocol.a

SRCS := \
	capture_socket.c \
	ctl_protocol.c \
	daemon_ctl_protocol.c \
	debug_overlay.c \
	fsutil.c \
	misc_ipc.c \
	monitor_geometry.c \
	paths.c \
	steam_paths.c \
	wallpaper_catalog.c

OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))
CJSON_OBJ := $(OBJ_DIR)/cJSON.o

.PHONY: build clean

build: $(LIB)

$(LIB): $(OBJS) $(CJSON_OBJ) | $(TARGET_DIR)
	rm -f $@
	ar rcs $@ $(OBJS) $(CJSON_OBJ)

$(OBJ_DIR)/%.o: %.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(CJSON_OBJ): ../vendor/cjson/cJSON.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(TARGET_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(LIB)
