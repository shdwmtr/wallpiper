#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NATIVE_DIR="$SCRIPT_DIR/native"
BUILD_DIR="$SCRIPT_DIR/../target/kde"

QT_PREFIX=""
if command -v qmake6 >/dev/null 2>&1; then
    QT_PREFIX="$(qmake6 -query QT_INSTALL_PREFIX)"
elif command -v qtpaths6 >/dev/null 2>&1; then
    QT_PREFIX="$(qtpaths6 --install-prefix)"
fi

if [ -z "$QT_PREFIX" ]; then
    echo "!! could not find qmake6/qtpaths6 to locate Qt's install prefix." >&2
fi

echo "==> configuring (Qt install prefix: ${QT_PREFIX:-<default>})"
cmake -S "$NATIVE_DIR" -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Release ${QT_PREFIX:+-DCMAKE_INSTALL_PREFIX="$QT_PREFIX"}

echo "==> building"
cmake --build "$BUILD_DIR" --parallel

STALE_DIR="/usr/local/lib/qt6/qml/dev/wallpiper"
if [ -d "$STALE_DIR" ] && [ "$QT_PREFIX" != "/usr/local" ]; then
    echo "==> removing stale plugin from a previous run that installed to the wrong prefix ($STALE_DIR)"
    sudo rm -rf "$STALE_DIR"
fi

echo "==> installing QML plugin (needs root for the system Qt QML dir)"
sudo cmake --install "$BUILD_DIR"

echo "==> installing/upgrading the Plasma Wallpaper KPackage"
if ! kpackagetool6 --type Plasma/Wallpaper --upgrade "$SCRIPT_DIR/extension" 2>/dev/null; then
    kpackagetool6 --type Plasma/Wallpaper --install "$SCRIPT_DIR/extension"
fi
