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

[Jump back to README](../README.md#usage5)
