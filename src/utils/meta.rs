use std::path::PathBuf;
use sha2::{Sha256, Digest};

#[derive(Clone)]
pub struct ImageLibrary {
    cache_dir: PathBuf,
}

impl Default for ImageLibrary {
    fn default() -> Self {
        ImageLibrary::new()
    }
}

impl ImageLibrary {
    pub fn new() -> Self {
        let cache_dir = Self::get_library();
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).ok();
        }
        Self {
            cache_dir,
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

    
}
