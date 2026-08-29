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

[Jump back to README](../README.md#usage5)
