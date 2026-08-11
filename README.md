# 🗜️ wallpiper 

A runtime-interpose translation layer for [Wallpaper Engine](https://www.wallpaperengine.io/en) on top of [Proton GE](https://github.com/gloriouseggroll/proton-ge-custom) to allow native Wallpaper Engine support on Linux.

It's exactly what you'd expect, opposed to existing libraries like [linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine) (arguably misled by AI), wallpiper runs Wallpaper Engine natively without re-inventing the wheel. 

Given Proton GE is capable of it, wallpiper natively supports all existing Wallpaper Engine user wallpapers. 

## Dependencies

Core dependencies, needed regardless of which compositor portal you're building:

- [Just](https://github.com/casey/just)
- Rust
- GNU GCC
- Make
- CMake

On Arch Linux:

```sh
$ pacman -S just rustup base-devel
```

**NOTE**: If you use another Linux distribution, please contribute docs to this list and all relevant sub items below.


Each portal has its own additional dependencies, build step, and install step. See its section
below.

## GNOME (Mutter) portal

Implemented as an in-process GObject-Introspection library driven by a GNOME Shell extension.

### Dependencies

- `meson` and `ninja`
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

- `cmake`
- ECM (extra-cmake-modules)
- Qt6 (`Core`, `Gui`, `Qml`, `Quick`) >= 6.7
- `egl`
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



## License

Apache-2.0 see [`LICENSE.md`](./LICENSE.md).
