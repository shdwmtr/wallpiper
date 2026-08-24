# wallpiper 

An ~100kb native translation layer for Wallpaper Engine on GNU/Linux based compositors. 
Wallpiper offers native support for Wallpaper Engine, without re-inventing the wheel. 

If you find this utility/tool useful, please consider giving it a star ⭐

<img width="3840" height="2160" alt="image" src="https://github.com/user-attachments/assets/04ad57fd-8430-4510-9c8b-7e3ae0c7071c" />

## Usage

> Wallpiper is NOT installed to `PATH` automatically. All binaries live at `./target/release` relative to the repository root. 

```sh
# check config
$ wallpiperctl check-config

# run wallpiperd (manages proton and wallpaper engine. YOU DO NOTHING)
$ WALLPIPER_PORTAL=portal WALLPIPER_*... wallpiperd
```

## Dependencies

* make
* c99 compiler
* proton (>= v11.0 was tested working)
* dbus
* vulkan-headers
* vulkan-icd-loader
* fontconfig (optional: for local-font overrides)

On Arch Linux:

```sh
$ pacman -S base-devel dbus vulkan-headers vulkan-icd-loader fontconfig
```

## Installation

### Wallpiper Core

Start by building the core of wallpiper

```sh
$ make build-core

# optionally install wallpiper to `%HOME/.local/lib/wallpiper/`
$ make install-wallpiperd
```

### Wallpiper Portals

Wallpiper supports the following DE/WM(s) through the following portals. Jump to whatever section is relevant to you. 

* [wallpiper-portal-gnome](#gnome-mutter-portal)
* [wallpiper-portal-kde](#kde-plasma-portal)
* [wallpiper-portal-hyprland](#hyprland-portal)
* [wallpiper-portal-sway](#sway-portal)
* [wallpiper-portal-cosmic](#cosmic-portal)
* [wallpiper-portal-i3](#i3wm-portal)

Pull requests are welcome for additional portals. You can also submit an issue report to suggest other portals.

## GNOME (Mutter) portal

Implemented as an in-process GObject-Introspection library driven by a GNOME Shell extension.

### Dependencies

* mutter
* gobject-introspection
* mesa
* libdrm
* libxrandr
* libx11

On Arch Linux:

```sh
$ pacman -S mutter gobject-introspection mesa libdrm libxrandr libx11
```

### Build

```sh
$ make build-gnome
$ make install-gnome
```

Log out and back in, then ensure the extension is enabled. 

## KDE (Plasma) portal

Implemented as a Qt Quick/QML plugin, installed as a Plasma 6 Wallpaper KPackage.

### Dependencies

- cmake
- extra-cmake-modules
- qt6-base
- qt6-declarative
- mesa (for EGL)
- libx11

On Arch Linux:

```sh
$ pacman -S cmake extra-cmake-modules qt6-base qt6-declarative mesa libx11
```

### Build

```sh
$ make build-kde
$ make install-kde
```

1. Open System Settings -> Wallpaper
2. Open the Wallpaper type dropdown at the top of the panel
3. Select Wallpiper

## Hyprland portal

Uses `wlr-layer-shell`. No shell extension, and no install step.

### Dependencies

* wayland
* wayland-protocols

On Arch Linux:

```sh
$ pacman -S wayland wayland-protocols
```

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

Uses `wlr-layer-shell`. No shell extension, and no install step.

### Dependencies

* wayland
* wayland-protocols

On Arch Linux:

```sh
$ pacman -S wayland wayland-protocols
```

### Build

```sh
$ make build-sway
```

## COSMIC portal

> [!WARNING]
> COSMIC's IPC has no query for the compositor's global cursor position, so this portal can't support
> cursor-reactive wallpapers.

Uses `wlr-layer-shell`. No shell extension, and no install step.

### Dependencies

* wayland
* wayland-protocols

On Arch Linux:

```sh
$ pacman -S wayland wayland-protocols
```

### Build

```sh
$ make build-cosmic
```

## i3wm portal

Interfaces with X11 directly. No shell extension, and no install step.

### Dependencies

* libxcb (including its `xcb-dri3` and `xcb-shm` extensions)

On Arch Linux:

```sh
$ pacman -S libxcb
```

### Build

```sh
$ make build-i3
```

## Environment Variables 
`wallpiperd` has no persistent configuration, all variability is mutable through environment variables. 

#### `WALLPIPER_PORTAL` (required)
  Which portal to use: `hyprland`, `sway`, `cosmic`, `i3`, `gnome`, `kde`

#### `WALLPIPER_STEAM_ROOT`
  **Default:** auto-detected (`~/.local/share/Steam`, `~/.steam/steam`, `~/.steam/root`, or the Flatpak path)

  Your Steam library root

#### `WALLPIPER_PROTON_BIN`
  **Default:** auto-detected under `compatibilitytools.d/*/proton` or `steamapps/common/Proton */proton`

  Path to the `proton` binary to run Wallpaper Engine with

#### `WALLPIPER_WE_EXE`
  **Default:** `$STEAM_ROOT/steamapps/common/wallpaper_engine/wallpaper64.exe`

  Path to Wallpaper Engine's executable

#### `WALLPIPER_TEMP_DIR`
  **Default:** `/tmp/wallpiper`

  Directory used for ephemeral, session-scoped files (control sockets, the Vulkan capture layer's search path, tracked renderer PIDs)

#### `WALLPIPER_RUNTIME_DIR`
  **Default:** `$XDG_STATE_HOME/wallpiper`, or `~/.local/state/wallpiper`

  Directory used for state that should persist across reboots (e.g. the applied-DPI marker)


## Command API

`wallpiperctl` is a control process for the `wallpiperd` daemon. The following commands require `wallpiperd` to be actively running with the same `WALLPIPER_RUNTIME_DIR`.

```sh
# In research, not yet implemented:
# wallpiperctl mute
# wallpiperctl unmute
# wallpiperctl volume <0-100>

$ wallpiperctl debug-on
$ wallpiperctl debug-off
$ wallpiperctl check-config
```

## Common Issues

### Black Screen/Can't open Wallpaper Engine

Likely an issue with your underling translation layer. Only GE-Proton-11 and Valve-Proton-11 have been tested working. 

All users experiencing this issue fixed it upgrading to a later version of Proton. 

## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md). If wallpiper is useful to you, a star helps others find it!

## License

MIT see [`LICENSE`](./LICENSE).
