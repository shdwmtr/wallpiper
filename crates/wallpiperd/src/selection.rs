use serde::{Deserialize, Serialize};

use crate::config;

fn default_volume() -> u8 {
    100
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Selection {
    pub file: String,
    pub location: String,
    #[serde(default = "default_volume")]
    pub volume: u8,
    #[serde(default)]
    pub muted: bool,
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

pub fn update_audio_state(volume: Option<u8>, muted: Option<bool>) {
    let Some(mut selection) = load_selection() else {
        return;
    };
    if let Some(volume) = volume {
        selection.volume = volume;
    }
    if let Some(muted) = muted {
        selection.muted = muted;
    }
    save_selection(&selection);
}
