use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("expected crates/wallpiperd to be two levels below the workspace root");
    let translation_layer = workspace_root.join("translation-layer");

    println!("cargo:rerun-if-changed={}", translation_layer.display());

    let status = Command::new("just")
        .arg("--justfile")
        .arg(translation_layer.join("justfile"))
        .status()
        .expect("failed to invoke `just` for translation-layer");

    if !status.success() {
        panic!("translation-layer build failed with {status}");
    }
}
