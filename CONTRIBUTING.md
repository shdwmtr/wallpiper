# Contributing to wallpiper

Thanks for your interest in contributing!

## Project Layout

- `protocol/`: the wire protocol shared between `wallpiperd`, `wallpiperctl`, and the portals
- `include/`: public headers shared across the tree (`wallpiper/`, `wallpiper-wl/`)
- `libs/`: vendored dependencies (`cjson`, `tcc`, `libsnare`, `vulkan`)
- `loader/`: Vulkan capture layer (`vk-layer-hook`), POSIX interpose shim (`elf`), and the Win32
  `dwmapi` shim (`coff`) used to hook into Wallpaper Engine running under Proton
- `programs/wallpiperd`: the daemon: process/renderer management, portal dispatch, config
- `programs/wallpiperctl`: the control CLI for a running daemon
- `programs/wallpiper-portal-*`: one directory per DE/WM integration

Each portal directory is self-contained with its own `Makefile` and dependencies; you generally
only need to build the one you're working on.

## Before Submitting a PR

- All additions must be formatted. Run `make fmt` before committing.
- Keep portal-specific code inside its own `programs/wallpiper-portal-*` directory. Shared Wayland
  utils live in `wallpiper-portal-wl-common`.
- `wallpiperd` stays the only thing that talks to the renderer/protocol/portal layers directly.
  DO NOT reach into `protocol/` or `loader/` internals from portal code.

## Adding a New Portal

Additional portal support is *MORE* than welcome. Start by taking a look at an existing
portal similar to your target. 

If you're unsure whether a compositor is a good fit before writing any code, open an issue first.
