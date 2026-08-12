use wallpiper_protocol::MonitorGeometry;

pub const WALLPAPER_ENGINE_APP_ID: &str = "431960";

pub const FALLBACK_MONITOR: MonitorGeometry = MonitorGeometry {
    x: 0,
    y: 0,
    width: 1920,
    height: 1080,
    logical_width: 1920,
    logical_height: 1080,
};

fn home_dir() -> String {
    std::env::var("HOME").expect("HOME is not set")
}

fn env_path(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn install_dir_result() -> Result<std::path::PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("could not resolve wallpiperd's own executable path: {e}"))?;
    exe.parent()
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| "wallpiperd executable has no parent directory".to_string())
}

pub fn install_dir() -> std::path::PathBuf {
    install_dir_result().unwrap_or_else(|e| panic!("{e}"))
}

fn steam_root_result() -> Result<String, String> {
    if let Some(root) = env_path("WALLPIPER_STEAM_ROOT") {
        return Ok(root);
    }
    let home = home_dir();
    let candidates = [
        format!("{home}/.local/share/Steam"),
        format!("{home}/.steam/steam"),
        format!("{home}/.steam/root"),
        format!("{home}/.var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ];
    candidates
        .iter()
        .find(|path| std::path::Path::new(path).is_dir())
        .cloned()
        .ok_or_else(|| {
            format!(
                "could not find a Steam install. set WALLPIPER_STEAM_ROOT to its path \
                 (checked {})",
                candidates.join(", ")
            )
        })
}

pub fn steam_root() -> String {
    steam_root_result().unwrap_or_else(|e| panic!("{e}"))
}

fn compatdata_result() -> Result<String, String> {
    Ok(format!(
        "{}/steamapps/compatdata/{WALLPAPER_ENGINE_APP_ID}",
        steam_root_result()?
    ))
}

pub fn compatdata() -> String {
    compatdata_result().unwrap_or_else(|e| panic!("{e}"))
}

pub fn workshop_content_dir_result() -> Result<String, String> {
    Ok(format!(
        "{}/steamapps/workshop/content/{WALLPAPER_ENGINE_APP_ID}",
        steam_root_result()?
    ))
}

fn we_exe_result() -> Result<String, String> {
    if let Some(exe) = env_path("WALLPIPER_WE_EXE") {
        return Ok(exe);
    }
    let path = format!(
        "{}/steamapps/common/wallpaper_engine/wallpaper64.exe",
        steam_root_result()?
    );
    if !std::path::Path::new(&path).is_file() {
        return Err(format!(
            "Wallpaper Engine executable not found at {path}. set WALLPIPER_WE_EXE to its path"
        ));
    }
    Ok(path)
}

pub fn we_exe() -> String {
    we_exe_result().unwrap_or_else(|e| panic!("{e}"))
}

fn proton_bin_result() -> Result<String, String> {
    if let Some(bin) = env_path("WALLPIPER_PROTON_BIN") {
        return Ok(bin);
    }
    let tools_dir = format!("{}/compatibilitytools.d", steam_root_result()?);
    let mut candidates: Vec<String> = std::fs::read_dir(&tools_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let proton = entry.path().join("proton");
            proton
                .is_file()
                .then(|| proton.to_string_lossy().into_owned())
        })
        .collect();
    candidates.sort_by(|a, b| {
        let a_ge = a.to_lowercase().contains("ge");
        let b_ge = b.to_lowercase().contains("ge");
        b_ge.cmp(&a_ge).then_with(|| b.cmp(a))
    });
    candidates.into_iter().next().ok_or_else(|| {
        format!(
            "no Proton build found under {tools_dir}. Install Proton GE, or set \
             WALLPIPER_PROTON_BIN to a proton binary's path"
        )
    })
}

pub fn proton_bin() -> String {
    proton_bin_result().unwrap_or_else(|e| panic!("{e}"))
}

pub fn runtime_dir() -> String {
    wallpiper_protocol::runtime_dir()
}

pub fn state_file_result() -> Result<String, String> {
    env_path("WALLPIPER_STATE_FILE").ok_or_else(|| {
        "WALLPIPER_STATE_FILE not set. export WALLPIPER_STATE_FILE=<path> naming where to \
         persist the selected wallpaper (e.g. WALLPIPER_STATE_FILE=$HOME/.local/share/wallpiper/last_selection.json)"
            .to_string()
    })
}

pub fn state_file() -> String {
    state_file_result().unwrap_or_else(|e| panic!("{e}"))
}

fn portal_name_result() -> Result<String, String> {
    std::env::var("WALLPIPER_PORTAL").map_err(|_| {
        "WALLPIPER_PORTAL not set. export WALLPIPER_PORTAL=<name> naming an installed \
         wallpiper-portal-<name> binary (e.g. WALLPIPER_PORTAL=hyprland)"
            .to_string()
    })
}

pub fn portal_name() -> String {
    portal_name_result().unwrap_or_else(|e| panic!("{e}"))
}

pub fn preload_paths() -> String {
    format!("{}/libwallpiper-preload.so", install_dir().display())
}

pub fn vk_layer_path() -> String {
    runtime_dir()
}

pub fn wine_prefix() -> String {
    format!("{}/pfx", compatdata())
}

pub fn wineserver_bin() -> String {
    let dir = std::path::Path::new(&proton_bin())
        .parent()
        .expect("proton binary path has no parent directory")
        .to_path_buf();
    format!("{}/files/bin/wineserver", dir.display())
}

fn report(label: &str, result: Result<String, String>) {
    match result {
        Ok(value) => println!("  {label}: {value}"),
        Err(msg) => println!("  {label}: ERROR {msg}"),
    }
}

pub fn describe() {
    println!("wallpiper configuration:");
    report(
        "install dir",
        install_dir_result().map(|p| p.display().to_string()),
    );
    report("steam root", steam_root_result());
    report("proton binary", proton_bin_result());
    report("wallpaper engine exe", we_exe_result());
    report("compatdata", compatdata_result());
    report("workshop content dir", workshop_content_dir_result());
    report("runtime dir (WALLPIPER_RUNTIME_DIR)", Ok(runtime_dir()));
    report("state file (WALLPIPER_STATE_FILE)", state_file_result());
    report("portal (WALLPIPER_PORTAL)", portal_name_result());
}
