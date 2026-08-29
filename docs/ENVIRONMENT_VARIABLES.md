# Environment Variables

## WALLPIPER_PORTAL (*)
**DESCRIPTION**: Which portal to use: hyprland, sway, cosmic, i3, gnome, kde.

## WALLPIPER_STEAM_ROOT
**DEFAULT**: auto-detected (`~/.local/share/Steam`, `~/.steam/steam`, `~/.steam/root`, or the Flatpak path)

**DESCRIPTION**: Your Steam library root.

## WALLPIPER_PROTON_BIN
**DEFAULT**: auto-detected under compatibilitytools.d/*/proton or steamapps/common/Proton */proton

**DESCRIPTION**: Path to the proton binary to run Wallpaper Engine with.

## WALLPIPER_WE_EXE
**DEFAULT**: `$WALLPIPER_STEAM_ROOT/steamapps/common/wallpaper_engine/wallpaper64.exe`

**DESCRIPTION**: Path to Wallpaper Engine's executable.

## WALLPIPER_TEMP_DIR
**DEFAULT**: /tmp/wallpiper

**DESCRIPTION**: Directory used for ephemeral, session-scoped files, control sockets, the Vulkan capture 
layer's search path, tracked renderer PIDs, etc.

## WALLPIPER_RUNTIME_DIR
**DEFAULT**: `$XDG_STATE_HOME/wallpiper`, or `~/.local/state/wallpiper`

**DESCRIPTION**: Directory used for state that should persist across reboots.

## WALLPIPER_WE_UI_SCALE_FACTOR
**DEFAULT**: unset (no scaling override)

**DESCRIPTION**: Forces N scale factor on Wallpaper Engine's properties-panel process. Not auto-detected.

## WALLPIPER_TRAY_OPTS
**DEFAULT**: native

**DESCRIPTION**: Controls how the Wallpaper Engine tray icon is translated.

**OPTIONS**: [`native`, `notray`, `passthrough`]
* `native`: over org.kde.StatusNotifierItem/dbusmenu. Supports all portals.
* `notray`: no tray rendered at all
* `passthrough`: pushes a raw legacy XEmbed tray icon, which only appears if your desktop runs a legacy tray host.
