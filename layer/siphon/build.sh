#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source ../../scripts/lib/cbuild.sh

CC="${CC:-cc}"
VULKAN_CFLAGS=($(pkg-config --cflags vulkan))
CFLAGS=(-std=gnu99 -Os -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE
    -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables
    -I../../protocol/include -I../../vendor/vulkan "${VULKAN_CFLAGS[@]}")

TARGET_DIR=../../target/release
OBJ_DIR=../../target/objects/siphon
SO="$TARGET_DIR/libVkLayer_wallpiper_capture.so"
PROTOCOL_LIB="$TARGET_DIR/libwallpiper-protocol.a"

SRCS=(
    capture.c
    device.c
    instance.c
    layer_entry.c
    logging.c
    process.c
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
    if ! dry_run && is_stale "$SO" "${objs[@]}" "$PROTOCOL_LIB"; then
        echo "cc $(basename "$SO")"
        "$CC" "${CFLAGS[@]}" -shared -Wl,--gc-sections -s -o "$SO" "${objs[@]}" "$PROTOCOL_LIB" -lpthread
    fi
}

do_clean() {
    rm -rf "$OBJ_DIR" "$SO"
}

case "${1:-build}" in
    build) do_build ;;
    clean) do_clean ;;
    *) echo "usage: $0 {build|clean}" >&2; exit 1 ;;
esac
