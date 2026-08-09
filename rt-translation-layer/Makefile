CC ?= cc
CFLAGS ?= -O2 -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE
DEPFLAGS := -MMD -MP
TARGET_DIR := ../target/release
OBJ_DIR := ../target/objects

SRCS := $(shell find we-wallpaper-change-hook we-swapchain-interpose -name '*.c')
OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))
DEPS := $(OBJS:.o=.d)
SO := $(TARGET_DIR)/libwallpiper-preload.so

.PHONY: all clean

all: $(SO)

$(TARGET_DIR):
	mkdir -p $@

$(SO): $(OBJS) | $(TARGET_DIR)
	$(CC) $(CFLAGS) -shared -o $@ $(OBJS) -ldl -lpthread

$(OBJ_DIR)/%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(DEPFLAGS) -c -o $@ $<

clean:
	rm -rf $(OBJ_DIR) $(SO)

-include $(DEPS)
