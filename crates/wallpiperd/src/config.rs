use wallpiper_protocol::MonitorGeometry;

pub const STEAM_ROOT: &str = "/home/shdw/.local/share/Steam";
pub const PROTON_BIN: &str =
    "/home/shdw/.local/share/Steam/compatibilitytools.d/Proton-GE Latest/proton";
pub const WE_EXE: &str =
    "/home/shdw/.local/share/Steam/steamapps/common/wallpaper_engine/wallpaper64.exe";
pub const COMPATDATA: &str = "/home/shdw/.local/share/Steam/steamapps/compatdata/431960";
pub const REPO_ROOT: &str = "/home/shdw/Development/wallpiper";
pub const STATE_DIR: &str = "/home/shdw/.local/share/wallpiper";
pub const STATE_FILE: &str = "/home/shdw/.local/share/wallpiper/last_selection.json";

pub const FALLBACK_MONITOR: MonitorGeometry = MonitorGeometry {
    x: 0,
    y: 0,
    width: 1920,
    height: 1080,
    logical_width: 1920,
    logical_height: 1080,
};

pub fn portal_name() -> String {
    std::env::var("WALLPIPER_PORTAL").unwrap_or_else(|_| {
        panic!(
            "WALLPIPER_PORTAL not set — export WALLPIPER_PORTAL=<name> naming an installed \
             wallpiper-portal-<name> binary (e.g. WALLPIPER_PORTAL=hyprland)"
        )
    })
}

pub fn preload_paths() -> String {
    format!("{REPO_ROOT}/target/release/libwallpiper-preload.so")
}

pub fn vk_layer_path() -> String {
    STATE_DIR.to_string()
}

pub fn wine_prefix() -> String {
    format!("{COMPATDATA}/pfx")
}

pub fn wineserver_bin() -> String {
    let dir = std::path::Path::new(PROTON_BIN)
        .parent()
        .expect("PROTON_BIN has no parent directory");
    format!("{}/files/bin/wineserver", dir.display())
}
