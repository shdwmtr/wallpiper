use crate::config::{REPO_ROOT, STATE_DIR};

pub const VK_CAPTURE_LAYER_NAME: &str = "VK_LAYER_wallpiper_capture";

pub fn write_vk_layer_manifest() {
    let _ = std::fs::create_dir_all(STATE_DIR);
    let library_path = format!("{REPO_ROOT}/target/release/libVkLayer_wallpiper_capture.so");
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
    let path = format!("{STATE_DIR}/{VK_CAPTURE_LAYER_NAME}.json");
    if let Err(e) = std::fs::write(&path, manifest) {
        println!("failed to write vk layer manifest at {path}: {e}");
    }
}
