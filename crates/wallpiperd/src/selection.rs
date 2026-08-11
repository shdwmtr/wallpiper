use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Serialize, Deserialize, Debug)]
pub struct Selection {
    pub file: String,
    pub location: String,
}

pub fn load_selection() -> Option<Selection> {
    let data = std::fs::read_to_string(config::state_file()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_selection(selection: &Selection) {
    let state_file = config::state_file();
    if let Some(parent) = std::path::Path::new(&state_file).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            println!("failed to create directory {}: {e}", parent.display());
            return;
        }
    }
    let Ok(data) = serde_json::to_string_pretty(selection) else {
        println!("failed to serialize selection");
        return;
    };
    if let Err(e) = std::fs::write(&state_file, data) {
        println!("failed to write selection to {state_file}: {e}");
    }
}
