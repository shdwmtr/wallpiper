CC ?= cc
XCB_CFLAGS := $(shell pkg-config --cflags xcb xcb-dri3 xcb-shm)
XCB_LIBS := $(shell pkg-config --libs xcb xcb-dri3 xcb-shm)
CFLAGS ?= -std=gnu99 -O2 -Wall -Wextra -fPIC -D_GNU_SOURCE -I../../protocol/include -I../../vendor/cjson $(XCB_CFLAGS)

TARGET_DIR := ../../target/release
OBJ_DIR := ../../target/objects/wallpiper-portal-i3
BIN := $(TARGET_DIR)/wallpiper-portal-i3
PROTOCOL_LIB := $(TARGET_DIR)/libwallpiper-protocol.a

SRCS := \
	main.c

OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))

.PHONY: build build-protocol clean

build: build-protocol $(BIN)

$(BIN): $(OBJS) $(PROTOCOL_LIB) | $(TARGET_DIR)
	$(CC) $(CFLAGS) -o $@ $(OBJS) $(PROTOCOL_LIB) -lpthread -lm $(XCB_LIBS)

$(OBJ_DIR)/%.o: %.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

build-protocol:
	$(MAKE) -C ../../protocol build

$(TARGET_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(BIN)
