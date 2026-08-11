use crate::config;

pub const VK_CAPTURE_LAYER_NAME: &str = "VK_LAYER_wallpiper_capture";

pub fn write_vk_layer_manifest() {
    let runtime_dir = config::runtime_dir();
    let _ = std::fs::create_dir_all(&runtime_dir);
    let library_path = format!(
        "{}/libVkLayer_wallpiper_capture.so",
        config::install_dir().display()
    );
    let manifest = format!(
        r#"{{
    "file_format_version" : "1.0.0",
    "layer" : {{
        "name": "{VK_CAPTURE_LAYER_NAME}",
        "type": "GLOBAL",
        "library_path": "{library_path}",
        "api_version": "1.1.0",
        "implementation_version": "1",
        "description": "Wallpiper frame capture layer"
    }}
}}
"#
    );
    let path = format!("{runtime_dir}/{VK_CAPTURE_LAYER_NAME}.json");
    if let Err(e) = std::fs::write(&path, manifest) {
        println!("failed to write vk layer manifest at {path}: {e}");
    }
}
