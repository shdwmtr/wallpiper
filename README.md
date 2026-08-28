# wallpiper 

A light weight [Wallpaper Engine](https://store.steampowered.com/app/431960/Wallpaper_Engine/) translation layer for GNU/Linux based compositors without re-inventing the wheel. Proton does most of the heavy lifting, while wallpiper re-implements/patches niche portions of the PE/COFF Windows API Wallpaper Engine needs. 
Wallpiper then metaphorically "pipes" your wallpaper (using zero-copy [dma-buf](https://docs.kernel.org/driver-api/dma-buf.html)) from an internal frame buffer to a desktop portal. No overhead. 

If you find this utility/tool useful, please consider giving it a star ⭐

[[video showcase]](https://streamable.com/3hbykh)

## Why? Other projects exist? 

It's simple. Wallpiper is a translation layer, not a re-implementation. It doesn't depend on Wallpaper Engine, or any program it could theoretically run. Think of it more like Proton itself. 
Other projects, such as [linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine), are re-implementations. This means Wallpaper Engine's private spec/codebase is being cloned/mirrorred (largely by LLM technology, which has a tendency to hallucinate).
By design, its far more unstable, and not actually Wallpaper Engine. 

> NOTE: Wallpiper is early in development. Although backed by proper design, you may face breaking issues.

## Core Dependencies

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

We welcome additional packaging submissions, all related packaging logic resides in `.github/packages`

### Arch Linux (AUR)

Get it from the [AUR](https://aur.archlinux.org/packages/wallpiper), or compile [manually below](#building). 


## Building from source

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
Pull requests are welcome for additional portals. You can also submit an issue report to suggest other portals.

* [wallpiper-portal-gnome](./docs/MUTTER.md)
* [wallpiper-portal-kde](./docs/PLASMA.md)
* [wallpiper-portal-hyprland](./docs/HYPRLAND.md)
* [wallpiper-portal-sway](./docs/SWAY.md)
* [wallpiper-portal-cosmic](./docs/COSMIC.md)
* [wallpiper-portal-i3](./docs/I3WM.md)

## Usage

When built from source, wallpiper is NOT installed to `PATH` automatically. All binaries live at `./build/release` relative to the repository root. 

```sh
# check config
$ wallpiperctl check-config

# run wallpiperd (manages proton and wallpaper engine. YOU DO NOTHING)
$ WALLPIPER_PORTAL=portal [WALLPIPER_*...] wallpiperd
```

## Environment Variables 
`wallpiperd` has no persistent configuration, all variability is mutable through environment variables. 

See [docs/ENVIRONMENT_VARIABLES.md](./docs/ENVIRONMENT_VARIABLES.md) or `wallpiperd --help/-h`.

## Command API

`wallpiperctl` is a control process for the `wallpiperd` daemon, and wallpaper-engine. 

See See [docs/COMMAND_API.md](./docs/COMMAND_API.md) or `wallpiperctl --help/-h`. 

## Troubleshooting

See [TROUBLESHOOTING.md](./TROUBLESHOOTING.md).

## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md). If wallpiper is useful to you, a star helps others find it!

## License

MIT see [`LICENSE`](./LICENSE).
