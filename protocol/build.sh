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
source ../scripts/lib/cbuild.sh

CC="${CC:-cc}"
CFLAGS=(-std=gnu99 -Os -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE
    -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables
    -Iinclude -I../vendor/cjson)

TARGET_DIR=../target/release
OBJ_DIR=../target/objects/wallpiper-protocol
LIB="$TARGET_DIR/libwallpiper-protocol.a"

SRCS=(
    capture_socket.c
    ctl_protocol.c
    daemon_ctl_protocol.c
    debug_overlay.c
    fsutil.c
    monitor_geometry.c
    paths.c
    steam_paths.c
)

do_build() {
    mkdir -p "$TARGET_DIR" "$OBJ_DIR"
    local objs=()
    for src in "${SRCS[@]}"; do
        obj="$OBJ_DIR/${src%.c}.o"
        compile_c "$CC" "$obj" "$src" "${CFLAGS[@]}"
        objs+=("$obj")
    done
    cjson_obj="$OBJ_DIR/cJSON.o"
    compile_c "$CC" "$cjson_obj" "../vendor/cjson/cJSON.c" "${CFLAGS[@]}"
    objs+=("$cjson_obj")
    archive_lib "$LIB" "${objs[@]}"
}

do_clean() {
    rm -rf "$OBJ_DIR" "$LIB"
}

case "${1:-build}" in
    build) do_build ;;
    clean) do_clean ;;
    *) echo "usage: $0 {build|clean}" >&2; exit 1 ;;
esac
