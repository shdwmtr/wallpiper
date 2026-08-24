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

TCC_ROOT := ../../vendor/tcc
TCC_BIN := $(TCC_ROOT)/x86_64-win32-tcc
LIBSNARE_DIR := ../../vendor/libsnare
override CFLAGS := -std=gnu11 -Wall -Wextra -shared -nostdlib -ffreestanding -fno-builtin -isystem $(TCC_ROOT)/include -I . -I win32 -I $(LIBSNARE_DIR)

TARGET_DIR := ../../target/release
DLL := $(TARGET_DIR)/dwmapi.dll

SRCS := \
	src/dllmain.c \
	win32/bootstrap.c \
	win32/imports.c \
	win32/libc.c \
	src/util.c \
	src/pe_iat.c \
	src/progman.c \
	src/menu.c \
	src/tray.c \
	src/ipc_triggers.c \
	src/cursor.c \
	src/dwm_exports.c \
	src/waitobj.c \
	src/spawn.c

.PHONY: build build-tcc clean clean-tcc

build: build-tcc | $(TARGET_DIR)
	$(TCC_BIN) $(CFLAGS) -o $(DLL) $(SRCS)

build-tcc:
	cd $(TCC_ROOT) && { [ -f config.mak ] || ./configure; } && { [ -x x86_64-win32-tcc ] || $(MAKE) cross-x86_64-win32; }

$(TARGET_DIR):
	mkdir -p $@

clean:
	rm -f $(DLL)

clean-tcc:
	cd $(TCC_ROOT) && $(MAKE) clean
