#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source ../../scripts/lib/cbuild.sh

CC="${CC:-cc}"
CFLAGS=(-std=gnu99 -Os -Wall -Wextra -D_GNU_SOURCE
    -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables
    -I../../protocol/include -I../../vendor/cjson)

TARGET_DIR=../../target/release
OBJ_DIR=../../target/objects/wallpiperctl
BIN="$TARGET_DIR/wallpiperctl"
PROTOCOL_LIB="$TARGET_DIR/libwallpiper-protocol.a"

SRCS=(
    main.c
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
        "$CC" "${CFLAGS[@]}" -Wl,--gc-sections -s -o "$BIN" "${objs[@]}" "$PROTOCOL_LIB" -lpthread
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
