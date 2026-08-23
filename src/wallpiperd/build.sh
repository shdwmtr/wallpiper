#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source ../../scripts/lib/cbuild.sh

CC="${CC:-cc}"
DBUS_CFLAGS=($(pkg-config --cflags dbus-1))
DBUS_LIBS=($(pkg-config --libs dbus-1))
CFLAGS=(-std=gnu99 -Os -Wall -Wextra -D_GNU_SOURCE
    -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables
    -I../../protocol/include -I../../vendor/cjson "${DBUS_CFLAGS[@]}")

TARGET_DIR=../../target/release
OBJ_DIR=../../target/objects/wallpiperd
BIN="$TARGET_DIR/wallpiperd"
PROTOCOL_LIB="$TARGET_DIR/libwallpiper-protocol.a"

SRCS=(
    cleanup.c
    commands.c
    config.c
    dwmapi_shim.c
    font_rename.c
    fonts.c
    main.c
    portal.c
    process.c
    renderer.c
    signals.c
    tray_dbus.c
    tray_files.c
    vk_layer.c
)

do_build() {
    ../../protocol/build.sh build
    mkdir -p "$TARGET_DIR" "$OBJ_DIR"
    local objs=()
    for src in "${SRCS[@]}"; do
        obj="$OBJ_DIR/${src%.c}.o"
        compile_c "$CC" "$obj" "$src" "${CFLAGS[@]}"
        objs+=("$obj")
    done
    if ! dry_run && is_stale "$BIN" "${objs[@]}" "$PROTOCOL_LIB"; then
        echo "cc $(basename "$BIN")"
        "$CC" "${CFLAGS[@]}" -Wl,--gc-sections -s -o "$BIN" "${objs[@]}" "$PROTOCOL_LIB" -lpthread -lm "${DBUS_LIBS[@]}"
    fi
}

do_clean() {
    rm -rf "$OBJ_DIR" "$BIN"
}

case "${1:-build}" in
    build) do_build ;;
    clean) do_clean ;;
    *) echo "usage: $0 {build|clean}" >&2; exit 1 ;;
esac
