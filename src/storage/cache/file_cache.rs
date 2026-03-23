use std::path::{Path, PathBuf};
use tokio::fs;
use crate::util::hash::md5_hex;

#[derive(Clone)]
pub struct FileCache {
    root: PathBuf,
}

impl FileCache {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }

    pub async fn get(&self, user_ns: &str, key: &str) -> anyhow::Result<Option<String>> {
        let path = self.key_path(user_ns, key);
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(path).await?;
        Ok(Some(data))
    }

    pub async fn put(&self, user_ns: &str, key: &str, value: &str) -> anyhow::Result<()> {
        let path = self.key_path(user_ns, key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, value).await?;
        Ok(())
    }

    pub async fn remove(&self, user_ns: &str, key: &str) -> anyhow::Result<()> {
        let path = self.key_path(user_ns, key);
        if path.exists() {
            fs::remove_file(path).await?;
        }
        Ok(())
    }

    pub async fn exists(&self, user_ns: &str, key: &str) -> bool {
        let path = self.key_path(user_ns, key);
        path.exists()
    }

    fn key_path(&self, user_ns: &str, key: &str) -> PathBuf {
        let name = md5_hex(key);
        self.root.join(user_ns).join(name).with_extension("txt")
    }
}
