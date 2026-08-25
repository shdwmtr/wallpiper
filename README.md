# wallpiper 

An 100kb native Wallpaper Engine translation layer for GNU/Linux based compositors, without re-inventing the wheel. Proton does most of the heavy lifting, while wallpiper re-implements/patches niche portions of the PE/COFF Windows API Wallpaper Engine needs. 

Wallpiper then metaphorically "pipes" your wallpaper (using zero-copy [dma-buf](https://docs.kernel.org/driver-api/dma-buf.html)) from an internal frame buffer to a desktop portal. No overhead. 

If you find this utility/tool useful, please consider giving it a star ⭐

https://github.com/user-attachments/assets/515e3a3e-0666-486c-960d-e3195a84f241

## Why? Other projects exist? 

It's simple. Wallpiper is a translation layer, not a re-implementation. It doesn't depend on Wallpaper Engine, or any program it could theoretically run. Think of it more like Proton itself. 
Other projects, such as [linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine), are re-implementations. This means Wallpaper Engine's private spec/codebase is being cloned/mirrorred (largely by AI).
By design, its far more unstable, and not actually Wallpaper Engine. 

## Usage

> When built from source, wallpiper is NOT installed to `PATH` automatically. All binaries live at `./target/release` relative to the repository root. 

```sh
# check config
$ wallpiperctl check-config

# run wallpiperd (manages proton and wallpaper engine. YOU DO NOTHING)
$ WALLPIPER_PORTAL=portal WALLPIPER_*... wallpiperd
```

## Dependencies

* wallpaper-engine
* proton (>= v11.0 was tested working)
* fontconfig (optional: for local-font overrides)
* vulkan-headers
* vulkan-icd-loader
* make
* dbus
* c99 compiler

On Arch Linux:

```sh
$ pacman -S base-devel dbus vulkan-headers vulkan-icd-loader fontconfig
```

## Packaging

### Arch Linux (AUR)

Get it from the [AUR](https://aur.archlinux.org/packages/wallpiper), or compile [manually below](#building). 

## Building from source

> [!IMPORTANT]
> It is highly recommended that you have a clean, never before ran installation of Wallpaper Engine before continuing.
> Running it through Steam, Proton, Wine, or other compatability tools may silently break your install before you even start.
>
> If you face any issues during install, this should be your first hunch.

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

#### `WALLPIPER_WE_UI_SCALE_FACTOR`
  **Default:** unset (no scaling override)

  Forces N scale factor on Wallpaper Engines properties-panel process. Not auto-detected.

#### `WALLPIPER_TRAY_OPTS`
  **Default:** `native`

  Controls how the Wallpaper Engine tray icon is translated: `native`, `notray`, `passthrough`

  - `native` over `org.kde.StatusNotifierItem`/`dbusmenu` (supports all portals)
  - `notray` no tray rendered at all. 
  - `passthrough` pushes a raw legacy XEmbed tray icon, which only appears if your desktop runs a legacy tray host.

## Command API

`wallpiperctl` is a control process for the `wallpiperd` daemon, and wallpaper-engine. 

```
usage: wallpiperctl <command>

daemon commands (require a running wallpiperd):
  debug-on | debug-off

wallpaper engine commands:
  pause | play   | stop
  next  | prev   | reset
  mute  | unmute | volume <0-100>
  set <path|workshop-id> [monitor; int; 0-indexed]
  prop <path|workshop-id> <name> <value> [monitor; int; 0-indexed]
  list

standalone commands:
  check-config
```

## Common Issues

### Black Screen/Can't open Wallpaper Engine

Likely an issue with your underling translation layer. Only GE-Proton-11 and Valve-Proton-11 have been tested working. 

All users experiencing this issue fixed it upgrading to a later version of Proton. 

## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md). If wallpiper is useful to you, a star helps others find it!

## License

MIT see [`LICENSE`](./LICENSE).
