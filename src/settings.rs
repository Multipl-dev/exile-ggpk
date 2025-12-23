use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub ggpk_path: Option<String>,
    pub recent_files: Vec<String>,
}

impl AppSettings {
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("settings.json") {
             if let Ok(settings) = serde_json::from_str(&content) {
                 return settings;
             }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("settings.json", content);
        }
    }
}
