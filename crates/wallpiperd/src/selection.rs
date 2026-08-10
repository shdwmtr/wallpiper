use serde::Deserialize;

use crate::config::STATE_FILE;

#[derive(Deserialize, Debug)]
pub struct Selection {
    pub file: String,
    pub location: String,
}

pub fn load_selection() -> Option<Selection> {
    let data = std::fs::read_to_string(STATE_FILE).ok()?;
    serde_json::from_str(&data).ok()
}
