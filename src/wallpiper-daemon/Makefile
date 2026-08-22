CC ?= cc
DBUS_CFLAGS := $(shell pkg-config --cflags dbus-1)
DBUS_LIBS := $(shell pkg-config --libs dbus-1)
CFLAGS ?= -std=gnu99 -Os -Wall -Wextra -D_GNU_SOURCE -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables -I../../protocol/include -I../../vendor/cjson $(DBUS_CFLAGS)

TARGET_DIR := ../../target/release
OBJ_DIR := ../../target/objects/wallpiper-daemon
BIN := $(TARGET_DIR)/wallpiper-daemon
PROTOCOL_LIB := $(TARGET_DIR)/libwallpiper-protocol.a

SRCS := \
	cleanup.c \
	commands.c \
	config.c \
	dwmapi_shim.c \
	font_rename.c \
	fonts.c \
	main.c \
	portal.c \
	process.c \
	renderer.c \
	signals.c \
	tray_dbus.c \
	tray_files.c \
	vk_layer.c

OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))

.PHONY: build build-protocol clean

build: build-protocol $(BIN)

$(BIN): $(OBJS) $(PROTOCOL_LIB) | $(TARGET_DIR)
	$(CC) $(CFLAGS) -Wl,--gc-sections -s -o $@ $(OBJS) $(PROTOCOL_LIB) -lpthread -lm $(DBUS_LIBS)

$(OBJ_DIR)/%.o: %.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

build-protocol:
	$(MAKE) -C ../../protocol build

$(TARGET_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(BIN)
