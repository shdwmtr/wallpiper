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

[Jump back to README](../#usage5)
