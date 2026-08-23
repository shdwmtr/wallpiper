# wallpiper 

An ~100kb native translation layer for Wallpaper Engine on GNU/Linux based compositors.

## Dependencies

* A C99 Compiler
* Valve's Proton (>= v11.0 was tested working)

## Installation

### Wallpiper Core

Start by building the core of wallpiper

```sh
$ ./build.sh build-core

# optionally install wallpiper to `%HOME/.local/lib/wallpiper/`
$ ./build.sh install-wallpiperd
```

### Wallpiper Portals

Wallpiper supports the following DE/WM(s) through the following portals. Jump to whatever section is relevant to you. 

* [wallpiper-portal-gnome](#gnome-mutter-portal)
* [wallpiper-portal-kde](#kde-plasma-portal)
* [wallpiper-portal-hyprland](#hyprland-portal)
* [wallpiper-portal-i3](#i3wm-portal)
* [wallpiper-portal-sway](#sway-portal)

Pull requests are welcome for additional portals. You can also submit an issue report to suggest other portals.

## GNOME (Mutter) portal

Implemented as an in-process GObject-Introspection library driven by a GNOME Shell extension.

### Dependencies

* mutter
* gobject-introspection
* mesa
* libdrm

On Arch Linux:

```sh
$ pacman -S mutter gobject-introspection mesa libdrm
```

### Build

```sh
$ ./build.sh build-gnome

# requires root 
$ ./build.sh install-gnome
```

Log out and back in, then ensure the extension is enabled. 

## KDE (Plasma) portal

Implemented as a Qt Quick/QML plugin, installed as a Plasma 6 Wallpaper KPackage.

### Dependencies

- cmake
- extra-cmake-modules
- qt6-base
- qt6-declarative

On Arch Linux:

```sh
$ pacman -S cmake extra-cmake-modules qt6-base qt6-declarative
```

### Build

```sh
$ ./build.sh build-kde

# requires root 
$ ./build.sh install-kde
```

1. Open System Settings -> Wallpaper
2. Open the Wallpaper type dropdown at the top of the panel
3. Select Wallpiper

## Hyprland portal

Uses `wlr-layer-shell`. No shell extension, and no install step.

### Dependencies

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ ./build.sh build-hyprland
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

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ ./build.sh build-sway
```

## i3wm portal

Interfaces with X11 directly. No shell extension, and no install step.

### Dependencies

None beyond the [core dependencies](#dependencies).

### Build

```sh
$ ./build.sh build-i3
```

## Usage

`wallpiperd` has no persistent configuration, all variability is mutable through environment variables. 

| Variable | Default | Purpose |
| --- | --- | --- |
| `WALLPIPER_PORTAL` | *(required)* | Which portal to use: `hyprland`, `sway`, `i3`, `gnome`, `kde` |
| `WALLPIPER_STATE_FILE` | *(required)* | Full path of the file to persist the selected wallpaper to (created if missing) |
| `WALLPIPER_STEAM_ROOT` | auto-detected (`~/.local/share/Steam`, `~/.steam/steam`, `~/.steam/root`, or the Flatpak path) | Your Steam library root |
| `WALLPIPER_PROTON_BIN` | auto-detected under `compatibilitytools.d/*/proton` or `steamapps/common/Proton */proton` | Path to the `proton` binary to run Wallpaper Engine with |
| `WALLPIPER_WE_EXE` | `$STEAM_ROOT/steamapps/common/`<br>`wallpaper_engine/wallpaper64.exe` | Path to Wallpaper Engine's executable |
| `WALLPIPER_TEMP_DIR` | `/tmp/wallpiper` | Directory used for ephemeral, session-scoped files (control sockets, the Vulkan capture layer's search path, tracked renderer PIDs) |
| `WALLPIPER_RUNTIME_DIR` | `$XDG_STATE_HOME/wallpiper`, or `~/.local/state/wallpiper` | Directory used for state that should persist across reboots (e.g. the applied-DPI marker) |

```sh
# Check what it resolved before launching for real
$ ./target/release/wallpiperctl check-config

# Example on hyprland
$ WALLPIPER_PORTAL=hyprland ./target/release/wallpiperd
```

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

## License

Apache-2.0 see [`LICENSE`](./LICENSE).
