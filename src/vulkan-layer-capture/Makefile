CC ?= cc
VULKAN_CFLAGS := $(shell pkg-config --cflags vulkan)
CFLAGS ?= -std=gnu99 -O2 -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE -I../../protocol/include -I../../vendor/vulkan $(VULKAN_CFLAGS)

TARGET_DIR := ../../target/release
OBJ_DIR := ../../target/objects/vulkan-layer-capture
SO := $(TARGET_DIR)/libVkLayer_wallpiper_capture.so
PROTOCOL_LIB := $(TARGET_DIR)/libwallpiper-protocol.a

SRCS := \
	capture.c \
	device.c \
	instance.c \
	layer_entry.c \
	logging.c \
	process.c

OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))

.PHONY: build build-protocol clean

build: build-protocol $(SO)

$(SO): $(OBJS) $(PROTOCOL_LIB) | $(TARGET_DIR)
	$(CC) $(CFLAGS) -shared -o $@ $(OBJS) $(PROTOCOL_LIB) -lpthread

$(OBJ_DIR)/%.o: %.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

build-protocol:
	$(MAKE) -C ../../protocol build

$(TARGET_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(SO)
