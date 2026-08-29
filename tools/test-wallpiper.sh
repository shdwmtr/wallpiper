#!/usr/bin/env bash

make build-core
make build-$1

export WALLPIPER_PORTAL=$1
export WALLPIPER_WE_UI_SCALE_FACTOR=1.2
export WALLPIPER_TRAY_OPTS=native

exec ./build/release/wallpiperd
