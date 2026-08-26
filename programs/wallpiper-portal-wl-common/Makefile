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
WAYLAND_SCANNER ?= wayland-scanner
WAYLAND_PROTOCOLS_DIR := $(shell pkg-config --variable=pkgdatadir wayland-protocols)
WAYLAND_CFLAGS := $(shell pkg-config --cflags wayland-client)

TARGET_DIR := ../../build/release
OBJ_DIR := ../../build/objects/wallpiper-portal-wl-common
GEN_DIR := $(OBJ_DIR)/generated
LIB := $(TARGET_DIR)/libwallpiper-portal-wl-common.a
CFLAGS ?= -std=gnu99 -Os -Wall -Wextra
CFLAGS += -fPIC -fvisibility=hidden -D_GNU_SOURCE -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables -I. -I../../include -I$(GEN_DIR) $(WAYLAND_CFLAGS)

SRCS := \
	buffers.c \
	layer_surface.c \
	registry.c \
	run.c \
	state.c

OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))

GEN_HEADERS := \
	$(GEN_DIR)/wlr-layer-shell-unstable-v1-client-protocol.h \
	$(GEN_DIR)/linux-dmabuf-v1-client-protocol.h \
	$(GEN_DIR)/viewporter-client-protocol.h \
	$(GEN_DIR)/xdg-shell-client-protocol.h

GEN_OBJS := \
	$(OBJ_DIR)/wlr-layer-shell-unstable-v1-protocol.o \
	$(OBJ_DIR)/linux-dmabuf-v1-protocol.o \
	$(OBJ_DIR)/viewporter-protocol.o \
	$(OBJ_DIR)/xdg-shell-protocol.o

.PHONY: build clean

build: $(LIB)

$(LIB): $(OBJS) $(GEN_OBJS) | $(TARGET_DIR)
	rm -f $@
	ar rcs $@ $(OBJS) $(GEN_OBJS)

$(OBJ_DIR)/%.o: %.c $(GEN_HEADERS) | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(GEN_DIR)/wlr-layer-shell-unstable-v1-client-protocol.h $(GEN_DIR)/wlr-layer-shell-unstable-v1-protocol.c &: protocols/wlr-layer-shell-unstable-v1.xml | $(GEN_DIR)
	$(WAYLAND_SCANNER) client-header $< $(GEN_DIR)/wlr-layer-shell-unstable-v1-client-protocol.h
	$(WAYLAND_SCANNER) private-code $< $(GEN_DIR)/wlr-layer-shell-unstable-v1-protocol.c

$(GEN_DIR)/linux-dmabuf-v1-client-protocol.h $(GEN_DIR)/linux-dmabuf-v1-protocol.c &: $(WAYLAND_PROTOCOLS_DIR)/stable/linux-dmabuf/linux-dmabuf-v1.xml | $(GEN_DIR)
	$(WAYLAND_SCANNER) client-header $< $(GEN_DIR)/linux-dmabuf-v1-client-protocol.h
	$(WAYLAND_SCANNER) private-code $< $(GEN_DIR)/linux-dmabuf-v1-protocol.c

$(GEN_DIR)/viewporter-client-protocol.h $(GEN_DIR)/viewporter-protocol.c &: $(WAYLAND_PROTOCOLS_DIR)/stable/viewporter/viewporter.xml | $(GEN_DIR)
	$(WAYLAND_SCANNER) client-header $< $(GEN_DIR)/viewporter-client-protocol.h
	$(WAYLAND_SCANNER) private-code $< $(GEN_DIR)/viewporter-protocol.c

$(GEN_DIR)/xdg-shell-client-protocol.h $(GEN_DIR)/xdg-shell-protocol.c &: $(WAYLAND_PROTOCOLS_DIR)/stable/xdg-shell/xdg-shell.xml | $(GEN_DIR)
	$(WAYLAND_SCANNER) client-header $< $(GEN_DIR)/xdg-shell-client-protocol.h
	$(WAYLAND_SCANNER) private-code $< $(GEN_DIR)/xdg-shell-protocol.c

$(OBJ_DIR)/wlr-layer-shell-unstable-v1-protocol.o: $(GEN_DIR)/wlr-layer-shell-unstable-v1-protocol.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(OBJ_DIR)/linux-dmabuf-v1-protocol.o: $(GEN_DIR)/linux-dmabuf-v1-protocol.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(OBJ_DIR)/viewporter-protocol.o: $(GEN_DIR)/viewporter-protocol.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(OBJ_DIR)/xdg-shell-protocol.o: $(GEN_DIR)/xdg-shell-protocol.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(TARGET_DIR) $(OBJ_DIR) $(GEN_DIR):
	mkdir -p $@

clean:
	rm -rf $(OBJ_DIR) $(LIB)
