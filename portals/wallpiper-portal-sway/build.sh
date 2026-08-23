#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source ../../scripts/lib/cbuild.sh

CC="${CC:-cc}"
WAYLAND_CFLAGS=($(pkg-config --cflags wayland-client))
WAYLAND_LIBS=($(pkg-config --libs wayland-client))
CFLAGS=(-std=gnu99 -Os -Wall -Wextra -fPIC -D_GNU_SOURCE
    -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables
    -I../../protocol/include -I../wallpiper-portal-wl-common/include
    -I../../target/objects/wallpiper-portal-wl-common/generated -I../../vendor/cjson
    "${WAYLAND_CFLAGS[@]}")

TARGET_DIR=../../target/release
OBJ_DIR=../../target/objects/wallpiper-portal-sway
BIN="$TARGET_DIR/wallpiper-portal-sway"
PROTOCOL_LIB="$TARGET_DIR/libwallpiper-protocol.a"
WL_COMMON_LIB="$TARGET_DIR/libwallpiper-portal-wl-common.a"

SRCS=(
    main.c
)

do_build() {
    ../../protocol/build.sh build
    ../wallpiper-portal-wl-common/build.sh build
    mkdir -p "$TARGET_DIR" "$OBJ_DIR"
    local objs=()
    for src in "${SRCS[@]}"; do
        obj="$OBJ_DIR/${src%.c}.o"
        compile_c "$CC" "$obj" "$src" "${CFLAGS[@]}"
        objs+=("$obj")
    done
    if ! dry_run && is_stale "$BIN" "${objs[@]}" "$PROTOCOL_LIB" "$WL_COMMON_LIB"; then
        echo "cc $(basename "$BIN")"
        "$CC" "${CFLAGS[@]}" -Wl,--gc-sections -s -o "$BIN" "${objs[@]}" "$WL_COMMON_LIB" "$PROTOCOL_LIB" -lpthread -lm "${WAYLAND_LIBS[@]}"
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
