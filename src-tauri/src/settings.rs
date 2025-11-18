use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub word_wrap_enabled: bool,
    pub word_wrap_length: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            word_wrap_enabled: true,
            word_wrap_length: 100,
        }
    }
}

impl Settings {
    pub fn load_from_path(path: &Path) -> Self {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(settings) = serde_json::from_str(&contents) {
                return settings;
            }
        }

        Self::default()
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;

        Ok(())
    }
}
