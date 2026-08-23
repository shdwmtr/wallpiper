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

TCC_ROOT=../../vendor/tcc
TCC_BIN="$TCC_ROOT/x86_64-win32-tcc"
LIBSNARE_DIR=../../vendor/libsnare
CFLAGS=(-std=gnu11 -Wall -Wextra -shared -nostdlib -ffreestanding -fno-builtin
    -isystem "$TCC_ROOT/include" -I . -I win32 -I "$LIBSNARE_DIR")

TARGET_DIR=../../target/release
DLL="$TARGET_DIR/dwmapi.dll"

SRCS=(
    src/dllmain.c
    win32/bootstrap.c
    win32/imports.c
    win32/libc.c
    src/util.c
    src/pe_iat.c
    src/progman.c
    src/menu.c
    src/tray.c
    src/ipc.c
    src/ipc_triggers.c
    src/cursor.c
    src/dwm_exports.c
    src/waitobj.c
    src/spawn.c
)

build_tcc() {
    (
        cd "$TCC_ROOT"
        [ -f config.mak ] || ./configure
        [ -x x86_64-win32-tcc ] || make cross-x86_64-win32
    )
}

do_build() {
    build_tcc
    mkdir -p "$TARGET_DIR"
    trace_compile "$TCC_BIN" "${CFLAGS[@]}" -o "$DLL" "${SRCS[@]}"
    dry_run && return 0
    "$TCC_BIN" "${CFLAGS[@]}" -o "$DLL" "${SRCS[@]}"
}

do_clean() {
    rm -f "$DLL"
}

do_clean_tcc() {
    make -C "$TCC_ROOT" clean
}

case "${1:-build}" in
    build) do_build ;;
    clean) do_clean ;;
    clean-tcc) do_clean_tcc ;;
    *) echo "usage: $0 {build|clean|clean-tcc}" >&2; exit 1 ;;
esac
