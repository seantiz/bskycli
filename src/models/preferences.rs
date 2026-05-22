use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesViewModel {
    pub hide_replies: bool,
    pub hide_replies_by_unfollowed: bool,
    pub hide_reposts: bool,
    pub hide_quote_posts: bool,
    pub notify_likes: bool,
    pub notify_reposts: bool,
    pub notify_follows: bool,
    pub notify_mentions: bool,
    pub notify_replies: bool,
    pub notify_quotes: bool,
    pub notify_starterpack_joins: bool,
    pub last_seen_at: Option<String>,
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
        PreferencesViewModel {
            hide_replies: false,
            hide_replies_by_unfollowed: false,
            hide_reposts: false,
            hide_quote_posts: false,
            notify_likes: true,
            notify_reposts: true,
            notify_follows: true,
            notify_mentions: true,
            notify_replies: true,
            notify_quotes: true,
            notify_starterpack_joins: true,
            last_seen_at: None,
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(config_path, content)
    }

    pub fn enabled_notifications(&self) -> Option<Vec<String>> {
        let mut reasons = Vec::new();
        if self.notify_likes { reasons.push("like".to_string()); }
        if self.notify_reposts { reasons.push("repost".to_string()); }
        if self.notify_follows { reasons.push("follow".to_string()); }
        if self.notify_mentions { reasons.push("mention".to_string()); }
        if self.notify_replies { reasons.push("reply".to_string()); }
        if self.notify_quotes { reasons.push("quote".to_string()); }
        if self.notify_starterpack_joins { reasons.push("starterpack-joined".to_string()); }

        if reasons.len() == 7 { None } else { Some(reasons) }
    }

    fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .expect("Couldn't find config directory")
            .join("bskycli")
            .join("preferences.toml")
    }
}
