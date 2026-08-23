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
source ../../../scripts/lib/cbuild.sh

CC="${CC:-cc}"
GIR_SCANNER="${GIR_SCANNER:-g-ir-scanner}"
GIR_COMPILER="${GIR_COMPILER:-g-ir-compiler}"
CSTD_FLAGS=(-std=gnu11 -Os -Wall -Wextra -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables)

MUTTER_PKGS=(libmutter-18 mutter-cogl-18 mutter-clutter-18 gobject-2.0 gio-unix-2.0 gbm egl libdrm xrandr x11)
GI_PKGS=("${MUTTER_PKGS[@]}" gobject-introspection-1.0)

LIB_CFLAGS=("${CSTD_FLAGS[@]}" -fPIC -Iinclude $(pkg-config --cflags "${GI_PKGS[@]}"))
LIB_LIBS=($(pkg-config --libs "${MUTTER_PKGS[@]}"))

TEST_PRODUCER_CFLAGS=("${CSTD_FLAGS[@]}" $(pkg-config --cflags gbm libdrm))
TEST_PRODUCER_LIBS=($(pkg-config --libs gbm libdrm))

GI_LIBS_RAW="$(pkg-config --libs "${GI_PKGS[@]}")"
GI_EXTRA_LIBS=($(printf '%s' "$GI_LIBS_RAW" | tr ' ' '\n' | sed -n 's/^-l//p'))
GI_EXTRA_LIB_DIRS=($(printf '%s' "$GI_LIBS_RAW" | tr ' ' '\n' | grep '^-L' || true))
GI_INCLUDE_PATH="$(pkg-config --variable=girdir gobject-introspection-1.0)"
MUTTER_GIR_DIR="$(pkg-config --variable=girdir libmutter-18)"

PREFIX="${PREFIX:-/usr}"
LIBDIR="${LIBDIR:-$PREFIX/lib}"
DESTDIR="${DESTDIR:-}"

TARGET_DIR=../../../target/gnome
OBJ_DIR="$TARGET_DIR/objects"

SO="$TARGET_DIR/libwallpiper-1.0.so"
TEST_PRODUCER="$TARGET_DIR/wallpiper-test-producer"
GIR="$TARGET_DIR/Wallpiper-1.0.gir"
TYPELIB="$TARGET_DIR/Wallpiper-1.0.typelib"

LIB_SRCS=(
    src/wallpiper_portal.c
    src/wallpiper_test_actor.c
    src/actor_stacking.c
    src/capture_listener.c
    src/ctl_listener.c
    src/egl_import.c
    src/error.c
    src/monitor_geometry.c
)

GIR_SRCS=(
    include/wallpiper_portal.h
    src/wallpiper_portal.c
    src/wallpiper_test_actor.c
)

do_build() {
    mkdir -p "$TARGET_DIR" "$OBJ_DIR"

    local lib_objs=()
    for src in "${LIB_SRCS[@]}"; do
        base="$(basename "$src")"
        obj="$OBJ_DIR/${base%.c}.o"
        compile_c "$CC" "$obj" "$src" "${LIB_CFLAGS[@]}"
        lib_objs+=("$obj")
    done
    if ! dry_run && is_stale "$SO" "${lib_objs[@]}"; then
        echo "cc $(basename "$SO")"
        "$CC" "${LIB_CFLAGS[@]}" -shared -Wl,--gc-sections -s -o "$SO" "${lib_objs[@]}" "${LIB_LIBS[@]}"
    fi

    local tp_obj="$OBJ_DIR/test_producer.o"
    compile_c "$CC" "$tp_obj" "tools/test_producer.c" "${TEST_PRODUCER_CFLAGS[@]}"
    if ! dry_run && is_stale "$TEST_PRODUCER" "$tp_obj"; then
        echo "cc $(basename "$TEST_PRODUCER")"
        "$CC" "${TEST_PRODUCER_CFLAGS[@]}" -Wl,--gc-sections -s -o "$TEST_PRODUCER" "$tp_obj" "${TEST_PRODUCER_LIBS[@]}"
    fi

    dry_run && return 0

    if is_stale "$GIR" "$SO" "${GIR_SRCS[@]}"; then
        echo "gir $(basename "$GIR")"
        "$GIR_SCANNER" --quiet --no-libtool \
            --namespace=Wallpiper --nsversion=1.0 \
            --identifier-prefix=Wallpiper --symbol-prefix=wallpiper \
            --warn-all \
            --include=GObject-2.0 \
            -Iinclude \
            --cflags-begin $(pkg-config --cflags "${GI_PKGS[@]}") --cflags-end \
            --add-include-path="$MUTTER_GIR_DIR" --add-include-path="$GI_INCLUDE_PATH" \
            -L"$TARGET_DIR" --library=wallpiper-1.0 \
            "${GI_EXTRA_LIB_DIRS[@]}" \
            "${GI_EXTRA_LIBS[@]/#/--extra-library=}" \
            --output "$GIR" \
            "${GIR_SRCS[@]}"
    fi

    if is_stale "$TYPELIB" "$GIR"; then
        echo "gic $(basename "$TYPELIB")"
        "$GIR_COMPILER" "$GIR" --output "$TYPELIB" --includedir="$MUTTER_GIR_DIR" --includedir="$GI_INCLUDE_PATH"
    fi
}

do_install() {
    do_build
    install -Dm755 "$SO" "$DESTDIR$LIBDIR/libwallpiper-1.0.so"
    install -Dm644 "$GIR" "$DESTDIR$PREFIX/share/gir-1.0/Wallpiper-1.0.gir"
    install -Dm644 "$TYPELIB" "$DESTDIR$LIBDIR/girepository-1.0/Wallpiper-1.0.typelib"
}

do_clean() {
    rm -rf "$TARGET_DIR"
}

case "${1:-build}" in
    build) do_build ;;
    install) do_install ;;
    clean) do_clean ;;
    *) echo "usage: $0 {build|install|clean}" >&2; exit 1 ;;
esac
