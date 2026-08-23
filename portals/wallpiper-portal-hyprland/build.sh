#!/usr/bin/env bash
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
OBJ_DIR=../../target/objects/wallpiper-portal-hyprland
BIN="$TARGET_DIR/wallpiper-portal-hyprland"
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
