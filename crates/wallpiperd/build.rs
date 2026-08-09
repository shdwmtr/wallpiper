use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("expected crates/wallpiperd to be two levels below the workspace root");
    let translation_layer = workspace_root.join("translation_layer");

    println!("cargo:rerun-if-changed={}", translation_layer.display());

    let status = Command::new("make")
        .arg("-C")
        .arg(&translation_layer)
        .status()
        .expect("failed to invoke `make` for translation_layer");

    if !status.success() {
        panic!("translation_layer build failed with {status}");
    }
}
