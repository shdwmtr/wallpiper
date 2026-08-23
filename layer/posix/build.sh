#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source ../../scripts/lib/cbuild.sh

CC="${CC:-cc}"
CFLAGS=(-std=gnu99 -Os -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE
    -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables)

TARGET_DIR=../../target/release
OBJ_DIR=../../target/objects
SO="$TARGET_DIR/libwallpiper-preload.so"

SRCS=(
    cursor_forward.c
    fonts.c
    guardian.c
    x11.c
)

do_build() {
    mkdir -p "$TARGET_DIR" "$OBJ_DIR"
    local objs=()
    for src in "${SRCS[@]}"; do
        obj="$OBJ_DIR/${src%.c}.o"
        compile_c "$CC" "$obj" "$src" "${CFLAGS[@]}"
        objs+=("$obj")
    done
    if ! dry_run && is_stale "$SO" "${objs[@]}"; then
        echo "cc $(basename "$SO")"
        "$CC" "${CFLAGS[@]}" -shared -Wl,--gc-sections -s -o "$SO" "${objs[@]}" -ldl -lpthread
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
