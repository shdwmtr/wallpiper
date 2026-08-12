# 🗜️ wallpiper 

A runtime-interpose translation layer for [Wallpaper Engine](https://www.wallpaperengine.io/en) on top of [Proton](https://github.com/valvesoftware/proton) to allow native Wallpaper Engine support on Linux.

It's exactly what you'd expect, opposed to existing libraries like [linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine) (arguably misled by AI), wallpiper runs Wallpaper Engine natively without re-inventing the wheel. 

Given Proton's translation layer holds, wallpiper natively supports all existing Wallpaper Engine wallpapers. 

## System Requirements

Compile time core dependencies, needed regardless of which compositor portal you're building. 

- Just
- Proton >= 11.0
- Rust >= 1.87
- GNU GCC >= 4.7 (C99, `__atomic` builtins)

On Arch Linux:

```sh
$ pacman -S just rustup base-devel
```

**NOTE**: If you use another Linux distribution, please contribute docs to this list and all relevant sub items below.

Each portal has its own additional dependencies, build step, and install step. See its section
below.

## Installation

Start by building the core of `wallpiper`

```sh
$ just build-core

# You can also install wallpiper and its related binaries to %HOME/.local/lib/wallpiper/*
# NOTE: This is not required, wallpiper can run from any directory. wallpiper assumes all its
# required binaries are in the same directory on disk (not cwd)
$ just install-wallpiperd
```

Now, you need to compile a relevant portal. Wallpiper supports the following DE/WM(s) through the following portals. Jump to whatever section is relevant to you. 

* [wallpiper-portal-gnome](#gnome-mutter-portal)
* [wallpiper-portal-kde](#kde-plasma-portal)
* [wallpiper-portal-hyprland](#hyprland-portal)
* [wallpiper-portal-i3](#i3wm-portal)
* [wallpiper-portal-sway](#sway-portal)

Pull requests are welcome for additional portals. You can also submit an issue report to suggest other portals.

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
Wayland, using `wlr-layer-shell`. No shell extension, and no install step.

### Dependencies

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ just build-hyprland
```

## Sway portal

A standalone Rust binary (`wallpiper-portal-sway`) that talks to the compositor directly over
Wayland, using `wlr-layer-shell`. No shell extension, and no install step.

sway's IPC has no query for the compositor's global cursor position, so this portal can't support
cursor-reactive wallpapers.

### Dependencies

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ just build-sway
```

## i3wm portal

A standalone Rust binary (`wallpiper-portal-i3`) that talks to the X server directly.
No shell extension, and no install step.

### Dependencies

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ just build-i3
```

## Usage

`wallpiperd` has no persistent configuration, all variability is mutable through environment variables. 

| Variable | Default | Purpose |
| --- | --- | --- |
| `WALLPIPER_PORTAL` | *(required)* | Which portal to use: `hyprland`, `sway`, `i3`, `gnome`, `kde` |
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

`wallpiperctl` is a control process for the `wallpiperd` daemon. The following commands require `wallpiperd` to be actively running with the same `WALLPIPER_RUNTIME_DIR`.

```sh
$ wallpiperctl set /path/to/wallpaper.pkg
$ wallpiperctl set --id <workshop_id>
$ wallpiperctl pause
$ wallpiperctl resume
$ wallpiperctl mute
$ wallpiperctl unmute
$ wallpiperctl volume <0-100>
$ wallpiperctl debug
$ wallpiperctl nodebug

# stateless, do not require wallpiperd to be running.
$ wallpiperctl list-wallpapers [-j]
$ wallpiperctl list-properties <workshop_id> [-j]
```

## Systemd

wallpiper as a systemd service is also a perfect fit. 

```ini
# ~/.config/systemd/user/wallpiperd.service
[Unit]
Description=wallpiper daemon
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
Environment=WALLPIPER_PORTAL=hyprland
Environment=WALLPIPER_STATE_FILE=%h/wallpiper.conf
ExecStart=%h/.local/lib/wallpiper/wallpiperd # assuming you've installed with `just install-wallpiperd`
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```

```sh
$ systemctl --user daemon-reload
$ systemctl --user enable --now wallpiperd.service
$ systemctl --user status wallpiperd.service
```

## License

Apache-2.0 see [`LICENSE.md`](./LICENSE.md).
