# 🗜️ wallpiper 

A runtime-interpose translation layer for [Wallpaper Engine](https://www.wallpaperengine.io/en) on top of [Proton GE](https://github.com/gloriouseggroll/proton-ge-custom) to allow native Wallpaper Engine support on Linux.

It's exactly what you'd expect, opposed to existing libraries like [linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine) (arguably misled by AI), wallpiper runs Wallpaper Engine natively without re-inventing the wheel. 

Given Proton GE is capable of it, wallpiper natively supports all existing Wallpaper Engine user wallpapers. 

## System Requirements

Compile time core dependencies, needed regardless of which compositor portal you're building. There are no runtime dependencies. 

- Just
- Rust >= 1.87
- GNU GCC >= 4.7 (C99, `__atomic` builtins)
- Make

On Arch Linux:

```sh
$ pacman -S just rustup base-devel
```

**NOTE**: If you use another Linux distribution, please contribute docs to this list and all relevant sub items below.

Each portal has its own additional dependencies, build step, and install step. See its section
below.

## Installation

Wallpiper supports the following DE/WM(s) through the following portals. Jump to whatever section is relevant to you. 

* [wallpiper-portal-gnome](#gnome-mutter-portal)
* [wallpiper-portal-kde](#kde-plasma-portal)
* [wallpiper-portal-hyprland](#hyprland-portal)

## GNOME (Mutter) portal

Implemented as an in-process GObject-Introspection library driven by a GNOME Shell extension.

### Dependencies

- Meson
- Ninja
- `libmutter-18`, `mutter-cogl-18`, `mutter-clutter-18`, `gobject-2.0`, `gio-unix-2.0`, `gbm`, `egl`, `libdrm`
- `gobject-introspection` (provides `g-ir-scanner`/`g-ir-compiler`, used to generate the typelib)

On Arch Linux:

```sh
$ pacman -S meson ninja mutter gobject-introspection mesa libdrm
```

### Build

```sh
$ just build-gnome
```

### Install

```sh
# requires root 
$ just install-gnome
```

Log out and back in (or restart GNOME Shell with `Alt`+`F2` -> `r` on X11), then ensure the extension is enabled. 

## KDE (Plasma) portal

Implemented as a Qt Quick/QML plugin, installed as a Plasma 6 Wallpaper KPackage.

### Dependencies

- cmake >= 3.16
- ECM (extra-cmake-modules)
- Qt6 (Core, Gui, Qml, Quick) >= 6.7
- EGL
- `kpackagetool6`, for installing the wallpaper plugin

On Arch Linux:

```sh
$ pacman -S cmake extra-cmake-modules qt6-base qt6-declarative
```

### Build

```sh
$ just build-kde
```

### Install

```sh
# requires root 
$ just install-kde
```

1. Open **System Settings** -> **Wallpaper**
2. Open the **Wallpaper type** dropdown at the top of the panel
3. Select **Wallpiper**

## Hyprland portal

A standalone Rust binary (`wallpiper-portal-hyprland`) that talks to the compositor directly over
Wayland. No shell extension, and no install step. 

### Dependencies

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ just build-cargo
```

## Launching Wallpiper

`wallpiperd` has no persistent configuration, all variability is mutable through environment variables. 

| Variable | Default | Purpose |
| --- | --- | --- |
| `WALLPIPER_PORTAL` | *(required)* | Which portal to use: `hyprland`, `gnome`, or `kde` |
| `WALLPIPER_STATE_FILE` | *(required)* | Full path of the file to persist the selected wallpaper to (created if missing) |
| `WALLPIPER_STEAM_ROOT` | auto-detected (`~/.local/share/Steam`, `~/.steam/steam`, `~/.steam/root`, or the Flatpak path) | Your Steam library root |
| `WALLPIPER_PROTON_BIN` | auto-detected, first `compatibilitytools.d/*/proton` with "GE" in the name, else any | Path to the `proton` binary to run Wallpaper Engine with |
| `WALLPIPER_WE_EXE` | `$STEAM_ROOT/steamapps/common/`<br>`wallpaper_engine/wallpaper64.exe` | Path to Wallpaper Engine's executable |
| `WALLPIPER_RUNTIME_DIR` | `/tmp/wallpiper` | Directory used for runtime files (e.g. the Vulkan capture layer's search path) |

```sh
# Check what it resolved before launching for real
$ ./target/release/wallpiperd check-config

# Example on hyprland
$ WALLPIPER_PORTAL=hyprland WALLPIPER_STATE_FILE=$HOME/wallpiper.conf ./target/release/wallpiperd
```

## Command API

### Runtime API

While `wallpiperd` is running, type commands into its stdin:

```
set /path/to/wallpaper.pkg
set --id <workshop_id>
pause
resume
mute
unmute
debug
nodebug
```

#### External Management

If you aren't programmatically managing wallpiper, you can export its stdin to FIFO to allow easy desktop usage

```sh
#!/usr/bin/env bash
mkfifo /tmp/wp.fifo 2>/dev/null || true
[ -p /tmp/wp.fifo ] || { echo "refusing to reuse non-fifo /tmp/wp.fifo" >&2; exit 1; }

tail -f /dev/null > /tmp/wp.fifo &
fifo_keeper=$!
trap 'kill $fifo_keeper 2>/dev/null' EXIT

WALLPIPER_PORTAL=hyprland WALLPIPER_STATE_FILE=$HOME/wallpiper.conf ./target/release/wallpiperd < /tmp/wp.fifo &
```

From an external shell, user, display, etc. 

```sh
# pause and resume wallpiper
$ echo "pause" > /tmp/wp.fifo
$ echo "resume" > /tmp/wp.fifo 
```

### CLI API

```sh
# API Declarations
list-wallpapers [-j|output as JSON]
list-properties <workshop_id> [-j|output as JSON]

# Examples
./target/release/wallpiperd list-wallpapers -j
./target/release/wallpiperd list-properties 2207463614 -j
```

## License

Apache-2.0 see [`LICENSE.md`](./LICENSE.md).
