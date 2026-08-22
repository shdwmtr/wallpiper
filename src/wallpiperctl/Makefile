CC ?= cc
CFLAGS ?= -std=gnu99 -Os -Wall -Wextra -D_GNU_SOURCE -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables -I../../protocol/include -I../../vendor/cjson

TARGET_DIR := ../../target/release
OBJ_DIR := ../../target/objects/wallpiperctl
BIN := $(TARGET_DIR)/wallpiperctl
PROTOCOL_LIB := $(TARGET_DIR)/libwallpiper-protocol.a

SRCS := \
	main.c

OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))

.PHONY: build build-protocol clean

build: build-protocol $(BIN)

$(BIN): $(OBJS) $(PROTOCOL_LIB) | $(TARGET_DIR)
	$(CC) $(CFLAGS) -Wl,--gc-sections -s -o $@ $(OBJS) $(PROTOCOL_LIB) -lpthread

$(OBJ_DIR)/%.o: %.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

build-protocol:
	$(MAKE) -C ../../protocol build

$(TARGET_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(BIN)
