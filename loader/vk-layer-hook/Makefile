#
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Ethan Alexander
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#

CC ?= cc
ARCH_SUFFIX ?=
M32_CFLAGS ?=
VULKAN_CFLAGS := $(shell pkg-config --cflags vulkan)
CFLAGS ?= -std=gnu99 -Os -Wall -Wextra
CFLAGS += -fPIC -fvisibility=hidden -D_GNU_SOURCE -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables -I../../include -I../../libs/vulkan $(VULKAN_CFLAGS) $(M32_CFLAGS)

TARGET_DIR := ../../build/release
OBJ_DIR := ../../build/objects/siphon$(ARCH_SUFFIX)
SO := $(TARGET_DIR)/libVkLayer_wallpiper_capture$(ARCH_SUFFIX).so
PROTOCOL_LIB := $(TARGET_DIR)/libwallpiper-protocol$(ARCH_SUFFIX).a

SRCS := \
	capture.c \
	device.c \
	instance.c \
	layer_entry.c \
	logging.c \
	process.c

OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))

.PHONY: build build32 build-protocol build-protocol32 clean

build: build-protocol $(SO)

build32:
	$(MAKE) ARCH_SUFFIX=32 M32_CFLAGS=-m32 build-protocol32
	$(MAKE) ARCH_SUFFIX=32 M32_CFLAGS=-m32 build

$(SO): $(OBJS) $(PROTOCOL_LIB) | $(TARGET_DIR)
	$(CC) $(CFLAGS) -shared -Wl,--gc-sections -s -o $@ $(OBJS) $(PROTOCOL_LIB) -lpthread

$(OBJ_DIR)/%.o: %.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

build-protocol:
	$(MAKE) -C ../../protocol build

build-protocol32:
	$(MAKE) -C ../../protocol build32

$(TARGET_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(SO)
