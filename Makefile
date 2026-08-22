KDE_BUILD_DIR := target/kde

.PHONY: help build build-core build-protocol build-daemon build-ctl build-vklayer \
        build-interpose build-dwmapi-shim build-wl-common build-hyprland build-i3 \
        build-sway build-gnome build-kde configure-kde install-gnome install-kde \
        install-wallpiperd compile-commands clean

help:
	@awk '/^[a-zA-Z0-9_-]+:/{sub(/:.*/, "", $$1); print $$1}' $(MAKEFILE_LIST) | sort -u

build-all: build-protocol build-daemon build-ctl build-vklayer build-interpose build-dwmapi-shim build-wl-common build-hyprland build-i3 build-sway build-gnome build-kde

build-core: build-protocol build-daemon build-ctl build-vklayer build-interpose build-dwmapi-shim

build-protocol:
	$(MAKE) -C protocol build

build-daemon:
	$(MAKE) -C src/wallpiperd build

build-ctl:
	$(MAKE) -C src/wallpiperctl build

build-vklayer:
	$(MAKE) -C layer/siphon build

build-interpose:
	$(MAKE) -C layer/posix build

build-dwmapi-shim:
	$(MAKE) -C layer/win32 build

build-wl-common:
	$(MAKE) -C portals/wallpiper-portal-wl-common build

build-hyprland:
	$(MAKE) -C portals/wallpiper-portal-hyprland build

build-i3:
	$(MAKE) -C portals/wallpiper-portal-i3 build

build-sway:
	$(MAKE) -C portals/wallpiper-portal-sway build

build-gnome:
	$(MAKE) -C portals/wallpiper-portal-gnome/native build

build-kde: configure-kde
	cmake --build $(KDE_BUILD_DIR) --parallel

configure-kde:
	cmake -S portals/wallpiper-portal-kde/native -B $(KDE_BUILD_DIR) -DCMAKE_BUILD_TYPE=Release

install-gnome: build-gnome
	sudo $(MAKE) -C portals/wallpiper-portal-gnome/native install
	sudo ldconfig
	mkdir -p "$(HOME)/.local/share/gnome-shell/extensions/wallpiper-gnome@wallpiper.dev"
	cp -r portals/wallpiper-portal-gnome/extension/. "$(HOME)/.local/share/gnome-shell/extensions/wallpiper-gnome@wallpiper.dev/"
	gnome-extensions enable wallpiper-gnome@wallpiper.dev

install-kde:
	./portals/wallpiper-portal-kde/install.sh

install-wallpiperd: build-core
	mkdir -p "$(HOME)/.local/lib/wallpiper"
	install -m 755 target/release/wallpiper-daemon "$(HOME)/.local/lib/wallpiper/"
	install -m 755 target/release/wallpiperctl "$(HOME)/.local/lib/wallpiper/"
	install -m 755 target/release/libwallpiper-preload.so "$(HOME)/.local/lib/wallpiper/"
	install -m 755 target/release/libVkLayer_wallpiper_capture.so "$(HOME)/.local/lib/wallpiper/"
	install -m 644 target/release/dwmapi.dll "$(HOME)/.local/lib/wallpiper/"
	for portal in hyprland i3 sway; do bin="target/release/wallpiper-portal-$$portal"; [ -f "$$bin" ] && install -m 755 "$$bin" "$(HOME)/.local/lib/wallpiper/" || true; done
	mkdir -p "$(HOME)/.local/bin"
	ln -sf "$(HOME)/.local/lib/wallpiper/wallpiper-daemon" "$(HOME)/.local/bin/wallpiper-daemon"
	ln -sf "$(HOME)/.local/lib/wallpiper/wallpiperctl" "$(HOME)/.local/bin/wallpiperctl"
	@echo "installed to $(HOME)/.local/lib/wallpiper, symlinked at $(HOME)/.local/bin/{wallpiper-daemon,wallpiperctl}"
	@echo "make sure $(HOME)/.local/bin is on your PATH"

compile-commands: configure-kde
	./scripts/gen-compile-commands.sh

clean:
	rm -rf target
