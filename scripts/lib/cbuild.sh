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


dry_run() {
    [ -n "${CBUILD_TRACE_DIR:-}" ]
}

is_stale() {
    local target="$1"
    shift
    [ ! -e "$target" ] && return 0
    local dep
    for dep in "$@"; do
        [ "$dep" -nt "$target" ] && return 0
    done
    return 1
}

trace_compile() {
    [ -z "${CBUILD_TRACE_DIR:-}" ] && return 0
    local f
    f="$(mktemp "$CBUILD_TRACE_DIR/rec.XXXXXX")"
    {
        pwd -P
        printf '%s\n' "$@"
    } >"$f"
}

compile_c() {
    local cc="$1" obj="$2" src="$3"
    shift 3
    trace_compile "$cc" "$@" -c -o "$obj" "$src"
    dry_run && return 0
    if is_stale "$obj" "$src"; then
        echo "cc $src"
        "$cc" "$@" -c -o "$obj" "$src"
    fi
}

archive_lib() {
    dry_run && return 0
    local lib="$1"
    shift
    if is_stale "$lib" "$@"; then
        echo "ar $(basename "$lib")"
        rm -f "$lib"
        ar rcs "$lib" "$@"
    fi
}
