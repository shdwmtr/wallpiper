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
WAYLAND_SCANNER="${WAYLAND_SCANNER:-wayland-scanner}"
WAYLAND_PROTOCOLS_DIR="$(pkg-config --variable=pkgdatadir wayland-protocols)"
WAYLAND_CFLAGS=($(pkg-config --cflags wayland-client))

TARGET_DIR=../../target/release
OBJ_DIR=../../target/objects/wallpiper-portal-wl-common
GEN_DIR="$OBJ_DIR/generated"
LIB="$TARGET_DIR/libwallpiper-portal-wl-common.a"
CFLAGS=(-std=gnu99 -Os -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE
    -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables
    -I. -Iinclude -I../../protocol/include -I"$GEN_DIR" "${WAYLAND_CFLAGS[@]}")

SRCS=(
    buffers.c
    layer_surface.c
    registry.c
    run.c
    state.c
)

PROTOCOLS=(
    "wlr-layer-shell-unstable-v1:protocols/wlr-layer-shell-unstable-v1.xml"
    "linux-dmabuf-v1:$WAYLAND_PROTOCOLS_DIR/stable/linux-dmabuf/linux-dmabuf-v1.xml"
    "viewporter:$WAYLAND_PROTOCOLS_DIR/stable/viewporter/viewporter.xml"
    "xdg-shell:$WAYLAND_PROTOCOLS_DIR/stable/xdg-shell/xdg-shell.xml"
)

generate_protocol() {
    local name="$1" xml="$2"
    local header="$GEN_DIR/${name}-client-protocol.h"
    local gen_src="$GEN_DIR/${name}-protocol.c"
    dry_run && return 0
    if is_stale "$header" "$xml" || is_stale "$gen_src" "$xml"; then
        echo "scan $xml"
        "$WAYLAND_SCANNER" client-header "$xml" "$header"
        "$WAYLAND_SCANNER" private-code "$xml" "$gen_src"
    fi
}

do_build() {
    mkdir -p "$TARGET_DIR" "$OBJ_DIR" "$GEN_DIR"

    local gen_headers=() gen_objs=()
    for entry in "${PROTOCOLS[@]}"; do
        name="${entry%%:*}"
        xml="${entry#*:}"
        generate_protocol "$name" "$xml"
        gen_headers+=("$GEN_DIR/${name}-client-protocol.h")
    done
    for entry in "${PROTOCOLS[@]}"; do
        name="${entry%%:*}"
        gen_src="$GEN_DIR/${name}-protocol.c"
        gen_obj="$OBJ_DIR/${name}-protocol.o"
        compile_c "$CC" "$gen_obj" "$gen_src" "${CFLAGS[@]}"
        gen_objs+=("$gen_obj")
    done

    local objs=()
    for src in "${SRCS[@]}"; do
        obj="$OBJ_DIR/${src%.c}.o"
        trace_compile "$CC" "${CFLAGS[@]}" -c -o "$obj" "$src"
        if ! dry_run && is_stale "$obj" "$src" "${gen_headers[@]}"; then
            echo "cc $src"
            "$CC" "${CFLAGS[@]}" -c -o "$obj" "$src"
        fi
        objs+=("$obj")
    done

    archive_lib "$LIB" "${objs[@]}" "${gen_objs[@]}"
}

do_clean() {
    rm -rf "$OBJ_DIR" "$LIB"
}

case "${1:-build}" in
    build) do_build ;;
    clean) do_clean ;;
    *) echo "usage: $0 {build|clean}" >&2; exit 1 ;;
esac
