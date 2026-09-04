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

KDE_BUILD_DIR := build/kde
XDG_DATA_HOME ?= $(HOME)/.local/share

.PHONY: help build-all build-core build-core32 build-protocol build-protocol32 build-daemon \
        build-ctl build-vklayer build-vklayer32 build-interpose build-interpose32 \
        build-dwmapi-shim build-dwmapi-shim32 build-wl-common build-hyprland build-i3 \
        build-sway build-cosmic build-gnome build-kde configure-kde install-gnome install-kde \
        install-wallpiperd compile-commands fmt clean

help:
	@awk '/^[a-zA-Z0-9_-]+:/{sub(/:.*/, "", $$1); print $$1}' $(MAKEFILE_LIST) | sort -u

build-all: build-protocol build-daemon build-ctl build-vklayer build-interpose build-dwmapi-shim build-wl-common build-hyprland build-i3 build-sway build-cosmic build-gnome build-kde

build-core: build-protocol build-daemon build-ctl build-vklayer build-interpose build-dwmapi-shim

build-core32: build-protocol32 build-vklayer32 build-interpose32 build-dwmapi-shim32

build-protocol:
	$(MAKE) -C protocol build

build-protocol32:
	$(MAKE) -C protocol build32

build-daemon: build-protocol
	$(MAKE) -C programs/wallpiperd build

build-ctl: build-protocol
	$(MAKE) -C programs/wallpiperctl build

build-vklayer: build-protocol
	$(MAKE) -C loader/vk-layer-hook build

build-vklayer32: build-protocol32
	$(MAKE) -C loader/vk-layer-hook build32

build-interpose:
	$(MAKE) -C loader/elf build

build-interpose32:
	$(MAKE) -C loader/elf build32

build-dwmapi-shim:
	$(MAKE) -C loader/coff build

build-dwmapi-shim32:
	$(MAKE) -C loader/coff build32

build-wl-common: build-protocol
	$(MAKE) -C programs/wallpiper-portal-wl-common build

build-hyprland: build-protocol build-wl-common
	$(MAKE) -C programs/wallpiper-portal-hyprland build

build-i3: build-protocol
	$(MAKE) -C programs/wallpiper-portal-i3 build

build-sway: build-protocol build-wl-common
	$(MAKE) -C programs/wallpiper-portal-sway build

build-cosmic: build-protocol build-wl-common
	$(MAKE) -C programs/wallpiper-portal-cosmic build

build-gnome:
	$(MAKE) -C programs/wallpiper-portal-gnome/native build

build-kde: configure-kde
	cmake --build $(KDE_BUILD_DIR) --parallel

configure-kde:
	cmake -S programs/wallpiper-portal-kde/native -B $(KDE_BUILD_DIR) -DCMAKE_BUILD_TYPE=Release

install-gnome: build-gnome
	sudo $(MAKE) -C programs/wallpiper-portal-gnome/native install
	sudo ldconfig
	mkdir -p "$(XDG_DATA_HOME)/gnome-shell/extensions/wallpiper-gnome@wallpiper.dev"
	cp -r programs/wallpiper-portal-gnome/extension/. "$(XDG_DATA_HOME)/gnome-shell/extensions/wallpiper-gnome@wallpiper.dev/"
	gnome-extensions enable wallpiper-gnome@wallpiper.dev

install-kde:
	./programs/wallpiper-portal-kde/install.sh

install-wallpiperd: build-core
	mkdir -p "$(HOME)/.local/lib/wallpiper"
	install -m 755 build/release/wallpiperd "$(HOME)/.local/lib/wallpiper/"
	install -m 755 build/release/wallpiperctl "$(HOME)/.local/lib/wallpiper/"
	install -m 755 build/release/libwallpiper-preload.so "$(HOME)/.local/lib/wallpiper/"
	install -m 755 build/release/libVkLayer_wallpiper_capture.so "$(HOME)/.local/lib/wallpiper/"
	install -m 644 build/release/dwmapi.dll "$(HOME)/.local/lib/wallpiper/"
	for f in libwallpiper-preload32.so libVkLayer_wallpiper_capture32.so; do [ -f "build/release/$$f" ] && install -m 755 "build/release/$$f" "$(HOME)/.local/lib/wallpiper/" || true; done
	[ -f build/release/dwmapi32.dll ] && install -m 644 build/release/dwmapi32.dll "$(HOME)/.local/lib/wallpiper/" || true
	for portal in hyprland i3 sway cosmic; do bin="build/release/wallpiper-portal-$$portal"; [ -f "$$bin" ] && install -m 755 "$$bin" "$(HOME)/.local/lib/wallpiper/" || true; done
	mkdir -p "$(HOME)/.local/bin"
	ln -sf "$(HOME)/.local/lib/wallpiper/wallpiperd" "$(HOME)/.local/bin/wallpiperd"
	ln -sf "$(HOME)/.local/lib/wallpiper/wallpiperctl" "$(HOME)/.local/bin/wallpiperctl"
	@echo "installed to $(HOME)/.local/lib/wallpiper, symlinked at $(HOME)/.local/bin/{wallpiperd,wallpiperctl}"
	@echo "make sure $(HOME)/.local/bin is on your PATH"

compile-commands: configure-kde
	./tools/gen-compile-commands.sh

fmt:
	find . \( -path ./libs -o -path ./build -o -path ./x86_64-pc-windows-tcc -o -path ./.git \) -prune -o \
		\( -name '*.c' -o -name '*.h' -o -name '*.cc' -o -name '*.cpp' -o -name '*.cxx' -o -name '*.hpp' -o -name '*.hh' \) -print0 \
		| xargs -0 -r clang-format -i

clean:
	rm -rf build
