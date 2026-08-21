# 🗜️ wallpiper 

A runtime-interpose translation layer for [Wallpaper Engine](https://www.wallpaperengine.io/en) on top of [Proton](https://github.com/valvesoftware/proton) to allow native Wallpaper Engine support on Linux.

It's exactly what you'd expect, opposed to existing libraries like [linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine) (arguably misled by AI), wallpiper runs Wallpaper Engine natively without re-inventing the wheel. 

Given Proton's translation layer holds, wallpiper natively supports all existing Wallpaper Engine wallpapers. 

## System Requirements

Compile time core dependencies, needed regardless of which compositor portal you're building. 

- GNU Make >= 4.3
- Proton >= 11.0
- GNU GCC >= 4.7 (C99, `__atomic` builtins)
- `pkg-config`
- `libxcb` (provides `xcb-dri3`, `xcb-shm`, used by the i3wm portal)
- `dbus` (used by the tray icon)
- `wayland`, `wayland-protocols` (used by the Hyprland/Sway portals)
- `vulkan-headers`

On Arch Linux:

```sh
$ pacman -S make base-devel pkgconf libxcb dbus wayland wayland-protocols vulkan-headers
```

**NOTE**: If you use another Linux distribution, please contribute docs to this list and all relevant sub items below.

Each portal has its own additional dependencies, build step, and install step. See its section
below.

## Installation

Start by building the core of `wallpiper`

```sh
$ make build-core

# You can also install wallpiper and its related binaries to %HOME/.local/lib/wallpiper/*
# NOTE: This is not required, wallpiper can run from any directory. wallpiper assumes all its
# required binaries are in the same directory on disk (not cwd)
$ make install-wallpiperd
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

- `libmutter-18`, `mutter-cogl-18`, `mutter-clutter-18`, `gobject-2.0`, `gio-unix-2.0`, `gbm`, `egl`, `libdrm`
- `gobject-introspection` (provides `g-ir-scanner`/`g-ir-compiler`, used to generate the typelib)

On Arch Linux:

```sh
$ pacman -S mutter gobject-introspection mesa libdrm
```

### Build

```sh
$ make build-gnome
```

### Install

```sh
# requires root 
$ make install-gnome
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
$ make build-kde
```

### Install

```sh
# requires root 
$ make install-kde
```

1. Open **System Settings** -> **Wallpaper**
2. Open the **Wallpaper type** dropdown at the top of the panel
3. Select **Wallpiper**

## Hyprland portal

A standalone C binary (`wallpiper-portal-hyprland`) that talks to the compositor directly over
Wayland, using `wlr-layer-shell`. No shell extension, and no install step.

### Dependencies

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ make build-hyprland
```

## Sway portal

> [!WARNING]
> Sway's IPC has no query for the compositor's global cursor position, so this portal can't support
> cursor-reactive wallpapers.
>
> * https://github.com/swaywm/sway/pull/8780
> * https://github.com/swaywm/sway/pull/8542
>
> I agree with the sway maintainer. Unfortunately, an after-thought-patch like this is not a proper solution
> and should not be merged.
>
> Solutions like https://github.com/cjacker/wl-find-cursor/ exist, however this is a single event library, not meant to be constantly
> driving mouse events. Mounting to `OVERLAY` instead of the `BACKGROUND` surface to actually intercept the mouse is not a proper solution.
> `OVERLAY` can't watch the cursor without also being the sole consumer. Sway would be unusable. 


A standalone C binary (`wallpiper-portal-sway`) that talks to the compositor directly over
Wayland, using `wlr-layer-shell`. No shell extension, and no install step.

### Dependencies

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ make build-sway
```

## i3wm portal

A standalone C binary (`wallpiper-portal-i3`) that talks to the X server directly.
No shell extension, and no install step.

### Dependencies

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ make build-i3
```

## Usage

`wallpiperd` has no persistent configuration, all variability is mutable through environment variables. 

| Variable | Default | Purpose |
| --- | --- | --- |
| `WALLPIPER_PORTAL` | *(required)* | Which portal to use: `hyprland`, `sway`, `i3`, `gnome`, `kde` |
| `WALLPIPER_STEAM_ROOT` | auto-detected (`~/.local/share/Steam`, `~/.steam/steam`, `~/.steam/root`, or the Flatpak path) | Your Steam library root |
| `WALLPIPER_PROTON_BIN` | auto-detected under `compatibilitytools.d/*/proton` or `steamapps/common/Proton */proton` | Path to the `proton` binary to run Wallpaper Engine with |
| `WALLPIPER_WE_EXE` | `$STEAM_ROOT/steamapps/common/`<br>`wallpaper_engine/wallpaper64.exe` | Path to Wallpaper Engine's executable |
| `WALLPIPER_TEMP_DIR` | `/tmp/wallpiper` | Directory used for ephemeral, session-scoped files (control sockets, the Vulkan capture layer's search path, tracked renderer PIDs) |
| `WALLPIPER_RUNTIME_DIR` | `$XDG_STATE_HOME/wallpiper`, or `~/.local/state/wallpiper` | Directory used for state that should persist across reboots (e.g. the applied-DPI marker) |
| `WALLPIPER_WEB_RENDERER_CEF_CMDLINE` | *(unset)* | Optional convenience for Wallpaper Engine's `steamuser.general.user.cefcommandline` setting in `config.json`, only applied if set |
| `WALLPIPER_FPS` | *(unset)* | Optional convenience for Wallpaper Engine's `steamuser.general.user.fps` setting in `config.json`, only applied if set |

`wallpiperd` always forces `steamuser.general.user.uihardwareacceleration` to `false` in Wallpaper Engine's `config.json` before spawning a renderer. `WALLPIPER_WEB_RENDERER_CEF_CMDLINE` and `WALLPIPER_FPS` are just convenience knobs for two more keys under that same `steamuser.general.user` object — all of these, including hardware acceleration, can also be set directly through Wallpaper Engine's own UI/settings (Settings → General), the env vars just save you from doing it by hand. Hardware-accelerated UI rendering is worth leaving off regardless of how you set it: it's a common source of crashes and rendering glitches under Proton, and disabling it costs little since it only affects Wallpaper Engine's own UI, not wallpaper playback.

```sh
# Check what it resolved before launching for real
$ ./target/release/wallpiperctl check-config

# Example on hyprland
$ WALLPIPER_PORTAL=hyprland ./target/release/wallpiper-daemon
```

## Command API

`wallpiperctl` is a control process for the `wallpiper-daemon` daemon. The following commands require `wallpiper-daemon` to be actively running with the same `WALLPIPER_TEMP_DIR`.

```sh
$ wallpiperctl set /path/to/wallpaper.pkg
$ wallpiperctl set --id <workshop_id>
$ wallpiperctl pause
$ wallpiperctl resume
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
ExecStart=%h/.local/lib/wallpiper/wallpiper-daemon # assuming you've installed with `make install-wallpiperd`
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
