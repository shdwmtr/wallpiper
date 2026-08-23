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

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KDE_CDB="$ROOT/target/kde/compile_commands.json"
OUT="$ROOT/compile_commands.json"
TARGET="${1:-build-all}"

compiler_re='^(cc|gcc|g\+\+|clang|clang\+\+|[A-Za-z0-9_.]+-gcc|[A-Za-z0-9_.]+-g\+\+|[A-Za-z0-9_.]+-clang|[A-Za-z0-9_.]+-clang\+\+)$'

json_escape() {
    local s=$1
    s=${s//\\/\\\\}
    s=${s//\"/\\\"}
    printf '%s' "$s"
}

is_compiler() {
    local base=${1##*/}
    [[ "$base" =~ $compiler_re ]] || [[ "$1" == *tcc* ]]
}

is_source() {
    [[ "$1" =~ \.(c|cc|cpp|cxx|m|mm)$ ]]
}

CBUILD_TRACE_DIR="$(mktemp -d)"
export CBUILD_TRACE_DIR
trap 'rm -rf "$CBUILD_TRACE_DIR"' EXIT

"$ROOT/build.sh" "$TARGET" >&2

declare -A entries

for f in "$CBUILD_TRACE_DIR"/rec.*; do
    [ -e "$f" ] || continue
    mapfile -t lines <"$f"
    [ "${#lines[@]}" -lt 2 ] && continue
    cwd="${lines[0]}"
    argv=("${lines[@]:1}")

    is_compiler "${argv[0]}" || continue

    sources=()
    for tok in "${argv[@]:1}"; do
        is_source "$tok" && sources+=("$tok")
    done
    [ "${#sources[@]}" -eq 0 ] && continue

    args_json=""
    for tok in "${argv[@]}"; do
        [ -n "$args_json" ] && args_json+=", "
        args_json+="\"$(json_escape "$tok")\""
    done

    for src in "${sources[@]}"; do
        key="${cwd}"$'\x1f'"${src}"
        entries["$key"]="{\"directory\": \"$(json_escape "$cwd")\", \"file\": \"$(json_escape "$src")\", \"arguments\": [$args_json]}"
    done
done

{
    printf '[\n'
    first=1
    for key in "${!entries[@]}"; do
        [ "$first" -eq 0 ] && printf ',\n'
        printf '%s' "${entries[$key]}"
        first=0
    done
    if [ -f "$KDE_CDB" ]; then
        [ "$first" -eq 0 ] && printf ',\n'
        sed -e '1d' -e '$d' "$KDE_CDB"
    fi
    printf '\n]\n'
} >"$OUT"

echo "wrote ${#entries[@]} build-derived entries (+ $([ -f "$KDE_CDB" ] && echo kde || echo no) kde entries) to $OUT" >&2
