#!/usr/bin/env bash

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
