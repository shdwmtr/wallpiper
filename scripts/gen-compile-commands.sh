#!/usr/bin/env bash
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

dirstack=("$ROOT")
declare -A entries

while IFS= read -r line; do
    if [[ "$line" =~ ^make(\[[0-9]+\])?:\ Entering\ directory\ \'(.+)\'$ ]]; then
        dirstack+=("${BASH_REMATCH[2]}")
        continue
    fi
    if [[ "$line" =~ ^make(\[[0-9]+\])?:\ Leaving\ directory\ \'(.+)\'$ ]]; then
        if [ "${#dirstack[@]}" -gt 1 ]; then
            dirstack=("${dirstack[@]:0:${#dirstack[@]}-1}")
        fi
        continue
    fi

    read -ra argv <<<"$line"
    [ "${#argv[@]}" -eq 0 ] && continue
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

    cwd="${dirstack[-1]}"
    for src in "${sources[@]}"; do
        key="${cwd}"$'\x1f'"${src}"
        entries["$key"]="{\"directory\": \"$(json_escape "$cwd")\", \"file\": \"$(json_escape "$src")\", \"arguments\": [$args_json]}"
    done
done < <(make -C "$ROOT" -n -w -B "$TARGET")

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

echo "wrote ${#entries[@]} make-derived entries (+ $([ -f "$KDE_CDB" ] && echo kde || echo no) kde entries) to $OUT" >&2
