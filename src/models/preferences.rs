use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesViewModel {
    pub hide_replies: bool,
    pub hide_replies_by_unfollowed: bool,
    pub hide_reposts: bool,
    pub hide_quote_posts: bool,
}

impl Default for PreferencesViewModel {
    fn default() -> Self {
        PreferencesViewModel {
            hide_replies: false,
            hide_replies_by_unfollowed: false,
            hide_reposts: false,
            hide_quote_posts: false,
        }
    }
}

impl PreferencesViewModel {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(prefs) = toml::from_str(&content) {
                    return prefs;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(config_path, content)
    }

    fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .expect("Couldn't find config directory")
            .join("bskycli")
            .join("preferences.toml")
    }
}
