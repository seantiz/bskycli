use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use sha2::{Sha256, Digest};

#[derive(Clone)]
pub struct ImageLibrary {
    cache_dir: PathBuf,
    memory: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl ImageLibrary {
    pub fn new() -> Self {
        let cache_dir = Self::get_library();
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).ok();
        }
        Self {
            cache_dir,
            memory: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_library() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("bskycli")
    }

    fn cache_path(&self, url: &str) -> PathBuf {
        let hash = format!("{:x}", Sha256::digest(url.as_bytes()));
        self.cache_dir.join(format!("{}.jpg", &hash[..16]))
    }

    pub async fn retrieve_or_download(&self, url: &str) -> std::io::Result<PathBuf> {
        let cache_path = self.cache_path(url);
        let url = url.to_string();

        if cache_path.exists() {
            return Ok(cache_path);
        }

        let client = reqwest::Client::new();
        let data = client
            .get(&url)
            .send()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?
            .bytes()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        std::fs::write(&cache_path, &data)?;
        Ok(cache_path)
    }

    pub fn clear_library(&self) -> std::io::Result<()> {
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                if let Ok(entry) = entry {
                    std::fs::remove_file(entry.path()).ok();
                }
            }
        }
        Ok(())
    }
}