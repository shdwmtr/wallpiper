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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NATIVE_DIR="$SCRIPT_DIR/native"
BUILD_DIR="$SCRIPT_DIR/../build/kde"

QT_PREFIX=""
if command -v qmake6 >/dev/null 2>&1; then
    QT_PREFIX="$(qmake6 -query QT_INSTALL_PREFIX)"
elif command -v qtpaths6 >/dev/null 2>&1; then
    QT_PREFIX="$(qtpaths6 --install-prefix)"
fi

if [ -z "$QT_PREFIX" ]; then
    echo "!! could not find qmake6/qtpaths6 to locate Qt's install prefix." >&2
fi

cmake -S "$NATIVE_DIR" -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Release ${QT_PREFIX:+-DCMAKE_INSTALL_PREFIX="$QT_PREFIX"}
cmake --build "$BUILD_DIR" --parallel

STALE_DIR="/usr/local/lib/qt6/qml/dev/wallpiper"
if [ -d "$STALE_DIR" ] && [ "$QT_PREFIX" != "/usr/local" ]; then
    echo "==> removing stale plugin from a previous run that installed to the wrong prefix ($STALE_DIR)"
    sudo rm -rf "$STALE_DIR"
fi

sudo cmake --install "$BUILD_DIR"

KPACKAGETOOL6=(kpackagetool6)
if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ]; then
    KPACKAGETOOL6=(sudo -u "$SUDO_USER" kpackagetool6)
fi

if ! "${KPACKAGETOOL6[@]}" --type Plasma/Wallpaper --upgrade "$SCRIPT_DIR/extension" 2>/dev/null; then
    "${KPACKAGETOOL6[@]}" --type Plasma/Wallpaper --install "$SCRIPT_DIR/extension"
fi
