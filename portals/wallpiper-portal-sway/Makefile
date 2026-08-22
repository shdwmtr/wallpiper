CC ?= cc
WAYLAND_CFLAGS := $(shell pkg-config --cflags wayland-client)
WAYLAND_LIBS := $(shell pkg-config --libs wayland-client)
CFLAGS ?= -std=gnu99 -Os -Wall -Wextra -fPIC -D_GNU_SOURCE -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables -I../../protocol/include -I../wallpiper-portal-wl-common/include -I../../target/objects/wallpiper-portal-wl-common/generated -I../../vendor/cjson $(WAYLAND_CFLAGS)

TARGET_DIR := ../../target/release
OBJ_DIR := ../../target/objects/wallpiper-portal-sway
BIN := $(TARGET_DIR)/wallpiper-portal-sway
PROTOCOL_LIB := $(TARGET_DIR)/libwallpiper-protocol.a
WL_COMMON_LIB := $(TARGET_DIR)/libwallpiper-portal-wl-common.a

SRCS := \
	main.c

OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))

.PHONY: build build-protocol build-wl-common clean

build: build-protocol build-wl-common $(BIN)

$(BIN): $(OBJS) $(PROTOCOL_LIB) $(WL_COMMON_LIB) | $(TARGET_DIR)
	$(CC) $(CFLAGS) -Wl,--gc-sections -s -o $@ $(OBJS) $(WL_COMMON_LIB) $(PROTOCOL_LIB) -lpthread -lm $(WAYLAND_LIBS)

$(OBJ_DIR)/%.o: %.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

build-protocol:
	$(MAKE) -C ../../protocol build

build-wl-common:
	$(MAKE) -C ../wallpiper-portal-wl-common build

$(TARGET_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(BIN)
