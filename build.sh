#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"
source scripts/lib/cbuild.sh

KDE_BUILD_DIR="target/kde"
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"

usage() {
    cat <<'EOF'
Usage: ./build.sh <target> [target...]

Targets:
  help                 list available targets
  build-all            build everything
  build-core           build daemon, ctl, and native capture layers
  build-protocol       build protocol library
  build-daemon         build wallpiper-daemon
  build-ctl            build wallpiperctl
  build-vklayer        build Vulkan capture layer
  build-interpose      build POSIX interpose layer
  build-dwmapi-shim    build Wine dwmapi shim
  build-wl-common      build shared Wayland portal code
  build-hyprland       build Hyprland portal
  build-i3             build i3 portal
  build-sway           build Sway portal
  build-gnome          build GNOME native module
  build-kde            configure and build KDE plugin
  configure-kde        run cmake configure for KDE plugin
  install-gnome        build and install the GNOME extension + native module
  install-kde          install the KDE plugin
  install-wallpiperd   build-core and install daemon/ctl/layers to ~/.local
  compile-commands     regenerate compile_commands.json
  clean                remove target/
EOF
}

build_protocol() { ./protocol/build.sh build; }
build_daemon() { ./src/wallpiperd/build.sh build; }
build_ctl() { ./src/wallpiperctl/build.sh build; }
build_vklayer() { ./layer/siphon/build.sh build; }
build_interpose() { ./layer/posix/build.sh build; }
build_dwmapi_shim() { ./layer/win32/build.sh build; }
build_wl_common() { ./portals/wallpiper-portal-wl-common/build.sh build; }
build_hyprland() { ./portals/wallpiper-portal-hyprland/build.sh build; }
build_i3() { ./portals/wallpiper-portal-i3/build.sh build; }
build_sway() { ./portals/wallpiper-portal-sway/build.sh build; }
build_gnome() { ./portals/wallpiper-portal-gnome/native/build.sh build; }

configure_kde() {
    cmake -S portals/wallpiper-portal-kde/native -B "$KDE_BUILD_DIR" -DCMAKE_BUILD_TYPE=Release
}

build_kde() {
    configure_kde
    dry_run && return 0
    cmake --build "$KDE_BUILD_DIR" --parallel
}

build_core() {
    build_protocol
    build_daemon
    build_ctl
    build_vklayer
    build_interpose
    build_dwmapi_shim
}

build_all() {
    build_core
    build_wl_common
    build_hyprland
    build_i3
    build_sway
    build_gnome
    build_kde
}

install_gnome() {
    build_gnome
    sudo ./portals/wallpiper-portal-gnome/native/build.sh install
    sudo ldconfig
    mkdir -p "$XDG_DATA_HOME/gnome-shell/extensions/wallpiper-gnome@wallpiper.dev"
    cp -r portals/wallpiper-portal-gnome/extension/. \
        "$XDG_DATA_HOME/gnome-shell/extensions/wallpiper-gnome@wallpiper.dev/"
    gnome-extensions enable wallpiper-gnome@wallpiper.dev
}

install_kde() {
    ./portals/wallpiper-portal-kde/install.sh
}

install_wallpiperd() {
    build_core
    mkdir -p "$HOME/.local/lib/wallpiper"
    install -m 755 target/release/wallpiperd "$HOME/.local/lib/wallpiper/"
    install -m 755 target/release/wallpiperctl "$HOME/.local/lib/wallpiper/"
    install -m 755 target/release/libwallpiper-preload.so "$HOME/.local/lib/wallpiper/"
    install -m 755 target/release/libVkLayer_wallpiper_capture.so "$HOME/.local/lib/wallpiper/"
    install -m 644 target/release/dwmapi.dll "$HOME/.local/lib/wallpiper/"
    for portal in hyprland i3 sway; do
        bin="target/release/wallpiper-portal-$portal"
        [ -f "$bin" ] && install -m 755 "$bin" "$HOME/.local/lib/wallpiper/" || true
    done
    mkdir -p "$HOME/.local/bin"
    ln -sf "$HOME/.local/lib/wallpiper/wallpiperd" "$HOME/.local/bin/wallpiperd"
    ln -sf "$HOME/.local/lib/wallpiper/wallpiperctl" "$HOME/.local/bin/wallpiperctl"
    echo "installed to $HOME/.local/lib/wallpiper, symlinked at $HOME/.local/bin/{wallpiperd,wallpiperctl}"
    echo "make sure $HOME/.local/bin is on your PATH"
}

compile_commands() {
    ./scripts/gen-compile-commands.sh
}

clean() {
    rm -rf target
}

run_target() {
    case "$1" in
        help) usage ;;
        build-all) build_all ;;
        build-core) build_core ;;
        build-protocol) build_protocol ;;
        build-daemon) build_daemon ;;
        build-ctl) build_ctl ;;
        build-vklayer) build_vklayer ;;
        build-interpose) build_interpose ;;
        build-dwmapi-shim) build_dwmapi_shim ;;
        build-wl-common) build_wl_common ;;
        build-hyprland) build_hyprland ;;
        build-i3) build_i3 ;;
        build-sway) build_sway ;;
        build-gnome) build_gnome ;;
        build-kde) build_kde ;;
        configure-kde) configure_kde ;;
        install-gnome) install_gnome ;;
        install-kde) install_kde ;;
        install-wallpiperd) install_wallpiperd ;;
        compile-commands) compile_commands ;;
        clean) clean ;;
        *)
            echo "unknown target: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
}

if [ "$#" -eq 0 ]; then
    usage
    exit 0
fi

for target in "$@"; do
    run_target "$target"
done
