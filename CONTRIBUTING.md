# Contributing to wallpiper

Thanks for your interest in contributing!

## Project Layout

- `protocol/`: the wire protocol shared between `wallpiperd`, `wallpiperctl`, and the portals
- `src/wallpiperd`: the daemon: process/renderer management, portal dispatch, config
- `src/wallpiperctl`: the control CLI for a running daemon
- `layer/`: Vulkan capture layer (`siphon`), POSIX interpose shim, and the Win32 `dwmapi` shim
  used to hook into Wallpaper Engine running under Proton
- `portals/wallpiper-portal-*`: one directory per DE/WM integration

Each portal directory is self-contained with its own `Makefile` and dependencies; you generally
only need to build the one you're working on.

## Before Submitting a PR

- All additions must be formatted. Run `make fmt` before committing.
- Keep portal-specific code inside its own `portals/wallpiper-portal-*` directory. Shared Wayland
  utils live in `wallpiper-portal-wl-common`.
- `wallpiperd` stays the only thing that talks to the renderer/protocol/portal layers directly.
  DO NOT reach into `protocol/` or `layer/` internals from portal code.

## Adding a New Portal

Additional portal support is *MORE* than welcome. Start by taking a look at an existing
portal similar to your target. 

If you're unsure whether a compositor is a good fit before writing any code, open an issue first.
