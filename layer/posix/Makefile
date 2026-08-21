CC ?= cc
CFLAGS ?= -std=gnu99 -O2 -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE

TARGET_DIR := ../../target/release
OBJ_DIR := ../../target/objects
SO := $(TARGET_DIR)/libwallpiper-preload.so

SRCS := \
	cursor_forward.c \
	fonts.c \
	guardian.c \
	x11.c

OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))

.PHONY: build clean

build: $(SO)

$(SO): $(OBJS) | $(TARGET_DIR)
	$(CC) $(CFLAGS) -shared -o $@ $(OBJS) -ldl -lpthread

$(OBJ_DIR)/%.o: %.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(TARGET_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(SO)
